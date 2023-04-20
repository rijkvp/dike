use crossbeam::channel::{unbounded, Sender};
use owo_colors::OwoColorize;
use std::{
    io::{stdout, Write},
    process::{Command, Output, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use tracing::debug;

use crate::report::Report;

pub trait Controller {
    fn get(&mut self) -> Option<CmdOptions>;
}

pub struct CmdOptions<'a> {
    cmd: &'a str,
    input: Option<String>,
    time_limit: Option<Duration>,
}

impl<'a> CmdOptions<'a> {
    pub fn new(cmd: &'a str, input: Option<String>, time_limit: Option<Duration>) -> CmdOptions {
        Self {
            cmd,
            input,
            time_limit,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CmdStatus {
    Killed(Option<String>),
    Terminated(CmdOutput),
}

impl CmdStatus {
    pub fn from_output(output: &Output, stdin: Option<String>, duration: Duration) -> Self {
        Self::Terminated(CmdOutput::from_output(output, duration, stdin))
    }
}

#[derive(Debug, Clone)]
pub struct CmdOutput {
    pub status: Option<i32>,
    pub duration: Duration,
    pub stdin: Option<String>,
    pub stdout: String,
    pub stderr: String,
}

impl CmdOutput {
    fn from_output(output: &Output, duration: Duration, stdin: Option<String>) -> Self {
        Self {
            status: output.status.code(),
            duration,
            stdin,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }

    pub fn print(&self) {
        if self.status != Some(0) {
            let exit_code = self
                .status
                .map(|s| s.to_string())
                .unwrap_or("(??)".to_string());
            println!(
                "{}",
                format_args!("Failed (exit code {}, took {:?})", exit_code, self.duration).red()
            );
        } else {
            println!("Took {:?}", self.duration.bright_green());
        }
        if let Some(stdin) = &self.stdin {
            println!("{} {}", "stdin:".bold(), stdin.trim_end());
        }
        if self.stdout.len() > 0 {
            println!("{} {}", "stdout:".bold(), self.stdout.trim_end());
        }
        if self.stderr.len() > 0 {
            println!("{} {}", "stderr:".bold(), self.stderr.trim_end());
        }
    }
}
pub fn run<T: Controller + Send + Sync + Clone + 'static>(
    controller: T,
    thread_count: u64,
    run_time: Option<Duration>,
) -> Report {
    let stop_signal = Arc::new(AtomicBool::new(false));
    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let (result_tx, result_rx) = unbounded::<CmdStatus>();

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

    let mut report = Report::new();
    let mut is_stopping = false;
    while !shutdown_signal.load(Ordering::Relaxed) {
        // Add new results to report
        for result in result_rx.try_iter() {
            report.insert(result);
        }

        print!(
            "\r{} {}",
            format!("Running ({}x)..", thread_count)
                .bright_magenta()
                .bold(),
            report.summary().blue()
        );
        stdout.flush().unwrap();

        if let Some(run_time) = run_time {
            if start_time.elapsed() > run_time && !is_stopping {
                stop_signal.store(true, Ordering::Relaxed);
                is_stopping = true;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    report
}

/// Spawns a worker thead controlled by the controller
fn spawn_worker<T: Controller + Send + Sync + 'static>(
    mut controller: T,
    result_sender: Sender<CmdStatus>,
    stop_signal: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        while !stop_signal.load(Ordering::Relaxed) {
            if let Some(cmd) = controller.get() {
                let result = run_command(cmd);
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
pub fn run_command(options: CmdOptions) -> CmdStatus {
    let start_time = Instant::now();
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(options.cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    if let Some(input) = &options.input {
        stdin
            .write_all(input.as_bytes())
            .expect("failed to write stdin");
        stdin.flush().expect("failed to flush stdin");
    }
    if let Some(time_limit) = options.time_limit {
        while let None = child.try_wait().unwrap() {
            if start_time.elapsed() > time_limit {
                child.kill().unwrap();
                return CmdStatus::Killed(options.input.clone());
            }
            thread::sleep(Duration::from_millis(50));
        }
    }
    let output = child.wait_with_output().unwrap();
    CmdStatus::from_output(&output, options.input.clone(), start_time.elapsed())
}
