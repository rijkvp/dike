mod error;
mod fuzz;
mod process;
mod test;
mod testfile;

use clap::{Parser, Subcommand};
use error::Error;
use fuzz::{FuzzConfig, FuzzInput};
use log::warn;
use std::{fs, path::PathBuf, process::Command, time::Duration};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    Test {
        test_file: PathBuf,
        command: String,
        #[clap(short = 'l')]
        time_limit: Option<f64>,
    },
    Fuzz {
        command: String,
        input: String,
        #[clap(short = 'o')]
        output: Option<PathBuf>,
        #[clap(short = 'l')]
        time_limit: Option<f64>,
        #[clap(short = 't')]
        threads: Option<u64>,
    },
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let args = Args::parse();

    match args.action {
        Action::Test {
            test_file: config,
            command,
            time_limit,
        } => {
            let file = fs::read_to_string(config)?;
            let testfile = file.parse()?;
            test::run_tests(command, testfile, time_limit.map(Duration::from_secs_f64))?;
        }
        Action::Fuzz {
            command,
            input,
            threads,
            time_limit,
            output: _,
        } => {
            let results = fuzz::run_fuzz(
                threads.unwrap_or_else(processor_count),
                FuzzConfig {
                    time_limit: time_limit.map(Duration::from_secs_f64),
                    command,
                    input: FuzzInput::parse(&input)?,
                },
                time_limit.map(Duration::from_secs_f64),
            );
            process::summarize_results(results);
        }
    }

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
