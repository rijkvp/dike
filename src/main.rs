mod error;
mod fuzzer;
mod result;
mod runner;
mod tester;
mod testfile;

use clap::{Parser, Subcommand};
use error::Error;
use fuzzer::Fuzzer;
use log::warn;
use std::{fs, path::PathBuf, process::Command, time::Duration};
use tester::Tester;
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
    /// Outputs results to a testfile
    #[clap(short = 'o', value_name = "FILE")]
    output: Option<PathBuf>,
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
    },
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let args = Args::parse();

    let thread_count = args.thread_count.unwrap_or(processor_count());
    let time_limit = args.time_limit.map(Duration::from_secs_f64);

    let results = match args.action {
        Action::Test {
            test_file: config,
            command,
        } => {
            let file = fs::read_to_string(config)?;
            let testfile = file.parse()?;
            let tester = Tester::new(
                testfile,
                command,
                time_limit.unwrap_or(Duration::from_secs(1)),
            );
            runner::run(tester, thread_count, None)
        }
        Action::Fuzz {
            command,
            input,
            run_time,
        } => {
            let fuzzer = Fuzzer::parse(&input, command, time_limit)?;
            let run_time = run_time.map(Duration::from_secs_f64);
            runner::run(fuzzer, thread_count, run_time)
        }
    };

    if let Some(output) = args.output {
        let testfile = TestFile::from_results(&results);
        fs::write(output, testfile.to_string())?;
    }

    result::summarize_results(&results);

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
