mod error;
mod fuzzer;
mod report;
mod runner;
mod test;
mod testfile;

use clap::{Parser, Subcommand};
use error::Error;
use fuzzer::Fuzzer;
use owo_colors::OwoColorize;
use report::{Report, TestReport};
use std::{path::PathBuf, process::Command, time::Duration};
use test::Tester;
use tracing::warn;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    #[clap(subcommand)]
    action: Action,
    /// Time limit for each run
    #[clap(short = 'l')]
    time_limit: Option<f64>,
    /// Sets a custom amount of threads
    #[clap(short = 't')]
    threads: Option<u64>,
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Args::command().debug_assert()
}

#[derive(Subcommand)]
enum Action {
    /// Tests a program with preset testcases
    Test {
        /// Directy that contains the tests
        test_dir: PathBuf,
        /// Command that runs the program to test
        command: String,
    },
    /// Test a program with randomly generated testcases
    Fuzz {
        /// Command that generates the inpu:w
        input_cmd: String,
        /// Command that runs the program to test
        cmd: String,
        /// Run for specified amount of seconds
        #[clap(short = 'r')]
        run_time: Option<f64>,
        /// Outputs results to a testfile
        #[clap(short = 'o', value_name = "FILE")]
        output: Option<PathBuf>,
    },
}

fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
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
    let time_limit = args.time_limit.map(Duration::from_secs_f64);

    match args.action {
        Action::Test { test_dir, command } => {
            let tests = testfile::read_tests(test_dir)?;
            // TODO: Don't clone here
            let tester = Tester::new(
                tests.clone(),
                command,
                time_limit.unwrap_or(Duration::from_secs(1)),
            );
            let results = runner::run(tester, thread_count, None);
            let report = Report::from_results(results);
            report.print_summary();
            let test_report = TestReport::from_report(report, tests);
            test_report.generate()?;
        }
        Action::Fuzz {
            cmd: command,
            input_cmd,
            run_time,
            output,
        } => {
            let fuzzer = Fuzzer::new(input_cmd, command, time_limit);
            let run_time = run_time.map(Duration::from_secs_f64);
            let results = runner::run(fuzzer, thread_count, run_time);
            if let Some(_output) = output {
                todo!("write output as test directory");
                //let testfile = TestFile::from_results(&results);
                //fs::write(output, testfile.to_string())?;
            }
            let report = Report::from_results(results);
            report.print_summary();
        }
    };
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
    warn!("Unable to get processor count. A default value of 1 will be used.");
    1
}
