use crossbeam::channel::{unbounded, Sender};
use log::debug;
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

pub trait Controller {
    fn get(&mut self) -> Option<RunCommand>;
}

pub struct RunCommand<'a> {
    pub command: &'a str,
    pub input: Option<String>,
    pub time_limit: Option<Duration>,
}

#[derive(Debug, Clone)]
pub enum ProcessResult {
    Unfinished(Option<String>),
    Finished(ProcessOutput),
}

impl ProcessResult {
    pub fn from_output(output: &Output, stdin: Option<String>, duration: Duration) -> Self {
        Self::Finished(ProcessOutput::from_output(output, duration, stdin))
    }
}

#[derive(Debug, Clone)]
pub struct ProcessOutput {
    pub status: Option<i32>,
    pub duration: Duration,
    pub stdin: Option<String>,
    pub stdout: String,
    pub stderr: String,
}

impl ProcessOutput {
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
                .unwrap_or("unkown".to_string());
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
    let mut is_stopping = false;
    while !shutdown_signal.load(Ordering::Relaxed) {
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

        if let Some(run_time) = run_time {
            if start_time.elapsed() > run_time && !is_stopping {
                stop_signal.store(true, Ordering::Relaxed);
                is_stopping = true;
            }
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
fn run_command(run: RunCommand) -> ProcessResult {
    let start_time = Instant::now();
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(run.command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    if let Some(input) = &run.input {
        stdin
            .write_all(input.as_bytes())
            .expect("failed to write stdin");
    }
    if let Some(time_limit) = run.time_limit {
        while let None = child.try_wait().unwrap() {
            if start_time.elapsed() > time_limit {
                child.kill().unwrap();
                return ProcessResult::Unfinished(run.input.clone());
            }
            thread::sleep(Duration::from_millis(50));
        }
    }
    let output = child.wait_with_output().unwrap();
    ProcessResult::from_output(&output, run.input.clone(), start_time.elapsed())
}
