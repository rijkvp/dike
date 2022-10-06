mod error;
mod fuzzer;
mod report;
mod runner;
mod test;
mod testfile;

use clap::{Parser, Subcommand};
use error::Error;
use fuzzer::{Fuzzer, InputTemplate};
use log::warn;
use owo_colors::OwoColorize;
use report::{Report, TestReport};
use std::{fs, path::PathBuf, process::Command, str::FromStr, time::Duration};
use test::Tester;
use testfile::TestFile;

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
    thread_count: Option<u64>,
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
        /// Command that runs the program to test
        command: String,
        /// File that specifies the tests
        test_file: PathBuf,
    },
    /// Tests a program with randomly generated testcases
    Fuzz {
        /// Command that runs the program to test
        command: String,
        /// Input template to specify the input format
        input: String,
        /// Run for specified amount of seconds
        #[clap(short = 'r')]
        run_time: Option<f64>,
        /// Outputs results to a testfile
        #[clap(short = 'o', value_name = "FILE")]
        output: Option<PathBuf>,
    },
}

fn main() {
    env_logger::init();
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

    let thread_count = args.thread_count.unwrap_or(processor_count());
    let time_limit = args.time_limit.map(Duration::from_secs_f64);

    match args.action {
        Action::Test {
            test_file: config,
            command,
        } => {
            let file = fs::read_to_string(config)?;
            let testfile = file.parse()?;
            let tester = Tester::new(
                &testfile,
                command,
                time_limit.unwrap_or(Duration::from_secs(1)),
            );
            let results = runner::run(tester, thread_count, None);
            let report = Report::from_results(results);
            report.print_summary();
            let test_report = TestReport::from_report(report, testfile.tests);
            test_report.print_summary();
        }
        Action::Fuzz {
            command,
            input,
            run_time,
            output,
        } => {
            let template = InputTemplate::from_str(&input)?;
            let fuzzer = Fuzzer::new(template, command, time_limit);
            let run_time = run_time.map(Duration::from_secs_f64);
            let results = runner::run(fuzzer, thread_count, run_time);
            if let Some(output) = output {
                let testfile = TestFile::from_results(&results);
                fs::write(output, testfile.to_string())?;
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
