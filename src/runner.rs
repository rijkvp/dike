use crate::{
    report::TestReport,
    tester::{TestExec, Tester},
};
use crossbeam_channel::{unbounded, Receiver, Sender};
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

#[derive(Debug, Clone)]
pub enum ExecResult {
    Timeout { test_id: usize },
    Terminated { test_id: usize, output: CmdOutput },
}

impl ExecResult {
    fn from_output(test_id: usize, output: Output, duration: Duration) -> Self {
        Self::Terminated {
            test_id,
            output: CmdOutput::from_output(output, duration),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CmdOutput {
    pub status: Option<i32>,
    pub duration: Duration,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl CmdOutput {
    fn from_output(output: Output, duration: Duration) -> Self {
        Self {
            status: output.status.code(),
            duration,
            stdout: output.stdout,
            stderr: output.stderr,
        }
    }
}

pub fn run(mut tester: Tester, thread_count: u64) -> TestReport {
    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let (exec_tx, exec_rx) = unbounded::<TestExec>();
    let (result_tx, result_rx) = unbounded::<ExecResult>();

    signal_hook::flag::register(signal_hook::consts::SIGINT, shutdown_signal.clone())
        .expect("Failed to register stop signal.");

    // Spawn worker threads
    let mut threads = Vec::new();
    for _ in 0..thread_count {
        let thread = spawn_worker(exec_rx.clone(), result_tx.clone());
        threads.push(thread);
    }

    let execs = tester.execs();
    thread::spawn(move || {
        for test_exec in execs {
            exec_tx.send(test_exec).unwrap();
        }
    });

    // Spawn a thread to wait for all workers to finish
    {
        let shutdown_signal = shutdown_signal.clone();
        thread::spawn(move || {
            for t in threads.into_iter() {
                t.join().unwrap();
            }
            shutdown_signal.store(true, Ordering::Relaxed);
        });
    }

    // Main thread prints progress and collects results
    let mut stdout = stdout();
    let test_count = tester.total_count();
    let mut result_count = 0;

    while !shutdown_signal.load(Ordering::Relaxed) && result_count < test_count {
        for result in result_rx.try_iter() {
            tester.report(result);
            result_count += 1;
        }

        print!(
            "\r{} {}",
            format!(
                "Running {}/{} tests ({}x)..",
                result_count, test_count, thread_count
            )
            .bright_magenta()
            .bold(),
            tester.summary().blue()
        );
        stdout.flush().unwrap();

        thread::sleep(Duration::from_millis(50));
    }
    println!();
    for result in result_rx.try_iter() {
        tester.report(result);
    }
    tester.into_report()
}

/// Spawns a worker thead controlled by the controller
fn spawn_worker(
    test_rceiver: Receiver<TestExec>,
    result_sender: Sender<ExecResult>,
) -> JoinHandle<()> {
    thread::spawn(move || loop {
        match test_rceiver.recv() {
            Ok(cmd) => {
                let result = run_command(cmd);
                if let Err(e) = result_sender.try_send(result) {
                    log::error!("Failed to send test result: {e}")
                };
            }
            Err(_) => break,
        }
    })
}

/// Runs the command and optionally writes input to stdin.
pub fn run_command(options: TestExec) -> ExecResult {
    let start_time = Instant::now();
    let mut child = Command::new(&options.cmd_args[0])
        .args(&options.cmd_args[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all(&options.input)
        .expect("failed to write stdin");
    stdin.flush().expect("failed to flush stdin");
    drop(stdin); // Close stdin to signal EOF
    if let Some(time_limit) = options.timeout {
        while let None = child.try_wait().unwrap() {
            if start_time.elapsed() > time_limit {
                child.kill().unwrap();
                return ExecResult::Timeout {
                    test_id: options.test_id,
                };
            }
            thread::sleep(Duration::from_millis(50));
        }
    }
    let output = child.wait_with_output().unwrap();
    ExecResult::from_output(options.test_id, output, start_time.elapsed())
}
