use crate::{
    error::Error,
    process::{self, ProcessResult},
    testfile::{TestCase, TestFile},
};
use owo_colors::OwoColorize;
use similar::{ChangeTag, TextDiff};
use std::{
    collections::HashMap,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

struct TestResult {
    id: u32,
    status: TestResultStatus,
}

enum TestResultStatus {
    Finished(ProcessResult),
    Unfinished,
}

pub fn run_tests(
    command: String,
    testfile: TestFile,
    time_limit: Option<Duration>,
) -> Result<(), Error> {
    let mut tests = HashMap::<u32, TestCase>::new();
    for (id, test) in testfile.tests.into_iter().enumerate() {
        tests.insert(id as u32, test);
    }

    // Spawn child threads
    let mut processes = Vec::<(u32, JoinHandle<ProcessResult>)>::new();
    for (id, test) in tests.iter() {
        let handle = {
            let command = command.clone();
            let input = test.input.clone();
            thread::spawn(move || process::run(&command, input))
        };
        processes.push((*id, handle));
    }

    // Join theads
    println!(
        "{}",
        format!("  Running {} tests..", processes.len()).yellow()
    );
    let start = Instant::now();
    let mut finished = Vec::<u32>::new();
    loop {
        for (id, process) in processes.iter_mut() {
            if !finished.contains(id) && process.is_finished() {
                finished.push(*id)
            }
        }
        let elapsed = start.elapsed();
        if let Some(time_limit) = time_limit {
            if elapsed >= time_limit {
                println!(
                    "{}",
                    format!(
                        "精 Time limit of {:.2}s exceeded.",
                        time_limit.as_secs_f32()
                    )
                    .red()
                );
                break;
            }
        }
        if finished.len() == tests.len() {
            println!(
                "{}",
                format!(
                    "祥 All tests finished in time limit (took {:.2}s total).",
                    elapsed.as_secs_f32()
                )
                .blue()
            );
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
        println!(
            "{} Result for {} {}",
            "==".dimmed(),
            test.name.white().bold(),
            "==".dimmed()
        );
        match result.status {
            TestResultStatus::Finished(result) => {
                if result.status != Some(0) {
                    let exit_code = {
                        if let Some(code) = result.status {
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
                    if !result.stderr.is_empty() {
                        println!("{}", result.stderr.trim().dimmed());
                    }
                    continue;
                }
                // if result.duration.as_secs_f32() < testfile.time {
                //     println!(
                //         "{}",
                //         format!(
                //             "  Finished in time limit of {:.2}s (took {:.2}s).",
                //             testfile.time,
                //             result.duration.as_secs_f32()
                //         )
                //         .green()
                //     );
                // } else {
                //     println!(
                //         "{}",
                //         format!(
                //             "  Exceeded time limit, took {:.2}s but the limit is {:.2}s.",
                //             result.duration.as_secs_f32(),
                //             testfile.time
                //         )
                //         .red()
                //     );
                // }
                if let Some(out) = &test.output {
                    if out == &result.stdout {
                        println!("{}", "  Correct standard output.".green());
                    } else {
                        println!(
                            "{}\n{}",
                            "  Wrong output, expected:".red(),
                            out.trim().dimmed()
                        );
                        if !result.stdout.is_empty() {
                            println!(
                                "{}\n{}",
                                "But received:".red(),
                                result.stdout.trim().dimmed()
                            );
                        } else {
                            println!("No output");
                        }
                    }
                }
                // if let Some(outfile) = &test.outfile {
                //     let outfile_contents = fs::read_to_string(outfile)?;
                //     if outfile_contents == result.stdout {
                //         println!("{}", "  Correct standard output (from file).".green());
                //     } else {
                //         println!("{}", "  Wrong output.".red(),);
                //         print_diff(&result.stdout, &outfile_contents);
                //     }
                // }
            }
            TestResultStatus::Unfinished => {
                println!("Unfinished.");
            }
        }
    }
    Ok(())
}

fn _print_diff(bad: &str, good: &str) {
    println!("{}", "  Output difference:".yellow());
    let diff = TextDiff::from_lines(bad, good);
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Delete => {
                print!(" {} {}", "-".red().bold(), change.dimmed().strikethrough())
            }
            ChangeTag::Insert => print!(" {} {}", "+".blue().bold(), change.blue()),
            ChangeTag::Equal => print!("   {}", change.green()),
        };
    }
}
