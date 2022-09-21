use clap::Parser;
use log::debug;
use owo_colors::OwoColorize;
use serde::Deserialize;
use similar::{ChangeTag, TextDiff};
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio},
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};
use thiserror::Error;

#[derive(Deserialize)]
struct TestConfig {
    tests: Vec<TestCase>,
    time: f32,
    time_limit: f32,
}

#[derive(Deserialize)]
struct TestCase {
    name: Option<String>,
    #[serde(rename = "in")]
    input: Option<String>,
    out: Option<String>,
    outfile: Option<PathBuf>,
}

struct TestResult {
    id: u32,
    status: TestResultStatus,
}

enum TestResultStatus {
    Finished(TestStats),
    Unfinished,
}

struct TestStats {
    duration: Duration,
    output: Output,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    config: PathBuf,
    command: String,
}

#[derive(Error, Debug)]
enum Error {
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Deserialization error: {0}")]
    Deserialize(#[from] serde_yaml::Error),
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let args = Args::parse();

    let file = fs::read_to_string(args.config)?;
    let cfg = serde_yaml::from_str::<TestConfig>(&file)?;

    let mut tests = HashMap::<u32, TestCase>::new();
    for (id, test) in cfg.tests.into_iter().enumerate() {
        tests.insert(id as u32, test);
    }

    // Spawn child threads
    let mut processes = Vec::<(u32, JoinHandle<TestStats>)>::new();
    for (id, test) in tests.iter() {
        processes.push((*id, spawn_test(args.command.clone(), test.input.clone())));
    }

    // Join theads
    println!("{}", format!("  Running {} tests..", processes.len()).yellow());
    let start = SystemTime::now();
    let mut finished = Vec::<u32>::new();
    loop {
        for (id, process) in processes.iter_mut() {
            if !finished.contains(id) && process.is_finished() {
                finished.push(*id)
            }
        }
        let elapsed= start.elapsed().unwrap().as_secs_f32();
        if elapsed >= cfg.time_limit {
            println!("{}", format!("精 Time limit of {:.2}s exceeded.", cfg.time_limit).red());
            break;
        }
        if finished.len() == tests.len() {
            println!("{}", format!("祥 All tests finished in time limit (took {:.2}s total).", elapsed).blue());
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    let mut results = Vec::<TestResult>::new();
    for (id, process) in processes.into_iter() {
        if finished.contains(&id) {
            let stats = process.join().unwrap();
            results.push(TestResult {
                id,
                status: TestResultStatus::Finished(stats),
            });
        } else {
            results.push(TestResult {
                id,
                status: TestResultStatus::Unfinished,
            });
        }
    }

    // Show results
    for result in results {
        let test = tests.get(&result.id).unwrap();
        let name = {
            if let Some(name) = test.name.clone() {
                name
            } else {
                format!("Test #{}", result.id)
            }
        };
        println!(
            "{} {}{} {}",
            "==".dimmed(),
            "Result for ",
            name.white().bold(),
            "==".dimmed()
        );
        match result.status {
            TestResultStatus::Finished(stats) => {
                let stdout = String::from_utf8_lossy(&stats.output.stdout);
                let stderr = String::from_utf8_lossy(&stats.output.stderr);
                if stats.output.status.code() != Some(0) {
                    let exit_code = {
                        if let Some(code) = stats.output.status.code() {
                            code.to_string()
                        } else {
                            "(unkown)".to_string()
                        }
                    };
                    println!(
                        "{}{}{}",
                        "  Execution failed (exit code ".red(),
                        exit_code.red().bold(),
                        ").".red()
                    );
                    if stderr.len() > 0 {
                        println!("{}", stderr.trim().dimmed());
                    }
                    continue;
                }
                if stats.duration.as_secs_f32() < cfg.time {
                    println!(
                        "{}",
                        format!(
                            "  Finished in time limit of {:.2}s (took {:.2}s).",
                            cfg.time,
                            stats.duration.as_secs_f32()
                        )
                        .green()
                    );
                } else {
                    println!(
                        "{}",
                        format!(
                            "  Exceeded time limit, took {:.2}s but the limit is {:.2}s.",
                            stats.duration.as_secs_f32(),
                            cfg.time
                        )
                        .red()
                    );
                }
                if let Some(out) = &test.out {
                    if out == &stdout {
                        println!("{}", "  Correct standard output.".green());
                    } else {
                        println!(
                            "{}\n{}",
                            "  Wrong output, expected:".red(),
                            out.trim().dimmed()
                        );
                        if stdout.len() > 0 {
                            println!("{}\n{}", "But received:".red(), stdout.trim().dimmed());
                        } else {
                            println!("No output");
                        }
                    }
                }
                if let Some(outfile) = &test.outfile {
                    let outfile_contents = fs::read_to_string(outfile)?;
                    if outfile_contents == stdout {
                        println!("{}", "  Correct standard output (from file).".green());
                    } else {
                        println!("{}", "  Wrong output.".red(),);
                        let stdout = stdout.to_string();
                        show_diff(&stdout, &outfile_contents);
                    }
                }
            }
            TestResultStatus::Unfinished => {
                println!("Unfinished.");
            }
        }
    }
    Ok(())
}

fn spawn_test(cmd: String, write_stdin: Option<String>) -> JoinHandle<TestStats> {
    let handle = thread::spawn(move || {
        let start_time = SystemTime::now();
        debug!("Spawning command '{cmd}'");
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        if let Some(input) = write_stdin {
            debug!("Writing '{input}' to stdin");
            stdin
                .write_all(input.as_bytes())
                .expect("failed to write stdin");
        }
        let output = child.wait_with_output().unwrap();
        let duration = start_time.elapsed().unwrap();
        debug!("Finished in {duration:?}");
        TestStats { duration, output }
    });
    handle
}

fn show_diff(bad: &str, good: &str) {
    println!("{}", "  Output difference:".yellow());
    let diff = TextDiff::from_lines(bad, good);
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Delete => {
                print!(" {} {}", "-".red().bold(), change.dimmed().strikethrough())
            }
            ChangeTag::Insert => print!(" {} {}", "+".blue().bold(), change.blue()),
            ChangeTag::Equal => print!(" {} {}", " ", change.green()),
        };
    }
}
