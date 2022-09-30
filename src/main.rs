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
use testfile::TestFile;
use std::{fs, path::PathBuf, process::Command, time::Duration};
use tester::Tester;

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    #[clap(subcommand)]
    action: Action,
    /// Stops running after specified amount of seconds
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
            let tester = Tester::new(testfile, command);
            runner::run(tester, thread_count, time_limit)
        }
        Action::Fuzz {
            command,
            input,
        } => {
            let fuzzer = Fuzzer::parse(&input, command)?;
            runner::run(fuzzer, thread_count, time_limit)
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
