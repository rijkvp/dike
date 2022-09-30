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

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    action: Action,
    #[clap(short = 'l')]
    time_limit: Option<f64>,
    #[clap(short = 't')]
    threads: Option<u64>,
}

#[derive(Subcommand)]
enum Action {
    Test {
        command: String,
        test_file: PathBuf,
    },
    Fuzz {
        command: String,
        input: String,
        #[clap(short = 'o')]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let args = Args::parse();

    let thread_count = args.threads.unwrap_or(processor_count());
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
            output: _,
        } => {
            let fuzzer = Fuzzer::parse(&input, command)?;
            runner::run(fuzzer, thread_count, time_limit)
        }
    };
    result::summarize_results(results);

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
