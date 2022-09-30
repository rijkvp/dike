use crate::result::ProcessResult;
use crossbeam::channel::{unbounded, Sender};
use log::debug;
use owo_colors::OwoColorize;
use std::{
    io::{stdout, Write},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub trait Controller {
    fn get(&mut self) -> Option<RunCommand>;
}

pub struct RunCommand {
    pub command: String,
    pub input: Option<String>,
}

pub fn run<T: Controller + Send + Sync + Clone + 'static>(
    controller: T,
    thread_count: u64,
    run_time: Option<Duration>,
) -> Vec<ProcessResult> {
    let stop_signal = Arc::new(AtomicBool::new(false));
    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let (result_tx, result_rx) = unbounded::<ProcessResult>();

    // Spawn worker threads
    let mut threads = Vec::new();
    for _ in 0..thread_count {
        let thread = spawn_worker(controller.clone(), result_tx.clone(), stop_signal.clone());
        threads.push(thread);
    }

    signal_hook::flag::register(signal_hook::consts::SIGINT, stop_signal.clone())
        .expect("Failed to register stop signal.");

    {
        let shutdown_signal = shutdown_signal.clone();
        thread::spawn(move || {
            for t in threads.into_iter() {
                t.join().unwrap();
            }
            shutdown_signal.store(true, Ordering::Relaxed);
        });
    }

    // Control on main thread
    let mut stdout = stdout();
    let start_time = Instant::now();
    let mut results = Vec::new();
    loop {
        let mut new_reports: Vec<ProcessResult> = result_rx.try_iter().collect();
        results.append(&mut new_reports);

        print!(
            "\r{} {}",
            format!("Running ({}x)..", thread_count)
                .bright_magenta()
                .bold(),
            format!(
                "{} {}",
                results.len().bright_blue().bold(),
                "results".blue()
            )
        );
        stdout.flush().unwrap();

        if let Some(time_limit) = run_time {
            if start_time.elapsed() > time_limit {
                println!("\n{}", "Time limit exceeded..".yellow());
                stop_signal.store(true, Ordering::Relaxed);
            }
        }
        if shutdown_signal.load(Ordering::Relaxed) {
            println!("\nShutdown signal received..");
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    results
}

/// Spawns a worker thead controlled by the controller
fn spawn_worker<T: Controller + Send + Sync + 'static>(
    mut controller: T,
    result_sender: Sender<ProcessResult>,
    stop_signal: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        while !stop_signal.load(Ordering::Relaxed) {
            if let Some(cmd) = controller.get() {
                let result = run_command(&cmd.command, cmd.input);
                if let Err(e) = result_sender.try_send(result) {
                    debug!("Failed to send report: {e}")
                };
            } else {
                break;
            }
        }
    })
}

/// Runs the command and optionally writes input to stdin.
fn run_command(cmd: &str, input: Option<String>) -> ProcessResult {
    let start_time = Instant::now();
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    if let Some(input) = &input {
        stdin
            .write_all(input.as_bytes())
            .expect("failed to write stdin");
    }
    let output = child.wait_with_output().unwrap();
    ProcessResult::from_output(&output, input, start_time.elapsed())
}
