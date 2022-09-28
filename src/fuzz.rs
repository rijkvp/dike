use crossbeam::channel::{unbounded, Sender};
use log::debug;
use owo_colors::OwoColorize;
use rand::Rng;
use std::{
    io::{stdout, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crate::{
    error::Error,
    process::{self, ProcessResult},
};

#[derive(Debug, Clone)]
pub struct FuzzInput {
    pub min: i64,
    pub max: i64,
}

impl FuzzInput {
    pub fn parse(string: &str) -> Result<Self, Error> {
        let spilts: Vec<&str> = string.split('-').collect();
        if spilts.len() != 2 {
            return Err(Error::Prase("Invalid count of '-'.".to_string()));
        }
        let min = spilts[0]
            .trim()
            .parse::<i64>()
            .map_err(|e| Error::Prase(format!("Failed to convert first part to string: {e}")))?;
        let max = spilts[1]
            .trim()
            .parse::<i64>()
            .map_err(|e| Error::Prase(format!("Failed to convert second part to string: {e}")))?;
        Ok(Self { min, max })
    }
    fn gen(&self) -> String {
        format!("{}\n", rand::thread_rng().gen_range(self.min..self.max))
    }
}

#[derive(Debug, Clone)]
pub struct FuzzConfig {
    pub command: String,
    pub time_limit: Option<Duration>,
    pub input: FuzzInput,
}

pub fn run_fuzz(
    thread_count: u64,
    config: FuzzConfig,
    run_time: Option<Duration>,
) -> Vec<ProcessResult> {
    let stop_signal = Arc::new(AtomicBool::new(false));
    let (result_tx, result_rx) = unbounded::<ProcessResult>();

    // Spawn worker threads
    let mut threads = Vec::new();
    for _ in 0..thread_count {
        let thread = spawn_worker(config.clone(), result_tx.clone(), stop_signal.clone());
        threads.push(thread);
    }

    signal_hook::flag::register(signal_hook::consts::SIGINT, stop_signal.clone())
        .expect("Failed to register stop signal.");

    // Control on main thread
    let mut stdout = stdout();
    let start_time = Instant::now();
    let mut results = Vec::new();
    while !stop_signal.load(Ordering::Relaxed) {
        let mut new_reports: Vec<ProcessResult> = result_rx.try_iter().collect();
        results.append(&mut new_reports);

        print!(
            "\r{} {}",
            format!("Fuzzing ({}x)..", thread_count)
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
        thread::sleep(Duration::from_millis(50));
    }
    println!("\n{}", "Shutting down..".yellow());
    for t in threads.into_iter() {
        t.join().unwrap();
    }
    results
}

fn spawn_worker(
    config: FuzzConfig,
    sender: Sender<ProcessResult>,
    stop_signal: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        while !stop_signal.load(Ordering::Relaxed) {
            let input = config.input.gen();
            let result = process::run(&config.command, Some(input));
            if let Err(e) = sender.try_send(result) {
                debug!("Failed to send report: {e}")
            };
        }
    })
}
