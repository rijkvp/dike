mod error;
mod loader;
mod report;
mod runner;
mod tester;

use clap::Parser;
use env_logger::Env;
use error::Error;
use owo_colors::OwoColorize;
use std::{process::Command, time::Duration};
use tester::Tester;

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    /// Command that runs the program to test
    cmd: String,
    /// Glob pattern for test files
    #[clap(short, long)]
    inputs: String,
    /// Time limit for each run
    #[clap(short = 'l', long)]
    timeout: Option<f64>,
    /// Sets a custom amount of threads, defaults to number of CPUs
    #[clap(short, long)]
    threads: Option<u64>,
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Args::command().debug_assert()
}

fn main() {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    if let Err(e) = run() {
        eprintln!(
            "{} {}",
            "Error:".bright_red().bold().underline(),
            e.to_string().white()
        );
    }
}

fn run() -> Result<(), Error> {
    let args = Args::parse();

    let thread_count = args.threads.unwrap_or(processor_count());
    let time_limit = args.timeout.map(Duration::from_secs_f64);

    let testscases = loader::load_tests(args.inputs)?;
    if testscases.is_empty() {
        log::warn!("No test cases found.");
        return Ok(());
    }
    let tester = Tester::new(testscases, args.cmd, time_limit);
    let report = runner::run(tester, thread_count);
    report.print_report();
    Ok(())
}

fn processor_count() -> u64 {
    // Run nproc to get system processors
    let output = Command::new("nproc").output().unwrap();
    if output.status.success() {
        if let Ok(threads) = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<u64>()
        {
            return threads;
        }
    }
    log::warn!("Unable to get processor count. A default value of 1 will be used.");
    1
}
