use crate::{
    runner::{ProcessOutput, ProcessResult},
    test::Test,
};
use owo_colors::OwoColorize;
use similar::{ChangeTag, TextDiff};
use std::collections::{HashMap, HashSet};

pub struct Report {
    unfinished: Vec<(u64, String)>,
    failed: Vec<ProcessOutput>,
    success: Vec<ProcessOutput>,
}

impl Report {
    pub fn from_results(results: Vec<ProcessResult>) -> Self {
        let mut unfinished = HashMap::<String, u64>::new();
        let mut failed = Vec::<ProcessOutput>::new();
        let mut success = Vec::<ProcessOutput>::new();
        for result in results.into_iter() {
            match result {
                ProcessResult::Unfinished(stdin) => {
                    if let Some(stdin) = stdin {
                        if let Some(val) = unfinished.get_mut(&stdin) {
                            *val += 1;
                        } else {
                            unfinished.insert(stdin.clone(), 1);
                        }
                    }
                }
                ProcessResult::Finished(output) => {
                    if output.status != Some(0) {
                        failed.push(output);
                    } else {
                        success.push(output);
                    }
                }
            }
        }
        let mut unfinished: Vec<(u64, String)> =
            unfinished.into_iter().map(|(k, v)| (v, k)).collect();
        unfinished.sort_by(|a, b| a.1.cmp(&b.1));
        Self {
            unfinished,
            failed,
            success,
        }
    }

    fn finished_count(&self) -> usize {
        self.success.len() + self.failed.len()
    }

    fn total_count(&self) -> usize {
        self.unfinished.len() + self.finished_count()
    }

    pub fn print_summary(&self) {
        println!(
            "\n\n{} {} {}",
            ">>".dimmed(),
            "Summary".bold().underline(),
            "<<".dimmed(),
        );

        print!("{} total", self.total_count());
        if self.failed.len() < self.total_count() {
            print!(
                ", {}/{} finished",
                self.finished_count().red(),
                self.total_count()
            );
        }
        if self.failed.len() > 0 {
            print!(", {}/{} failed", self.failed.len(), self.finished_count());
        }
        print!("\n");

        if self.unfinished.len() > 0 {
            println!(
                "\n{}",
                format!("神 Unfinished ({}x)", self.unfinished.len())
                    .yellow()
                    .bold()
            );
            for (count, input) in self.unfinished.iter() {
                println!("{}x {}", count, input.trim());
            }
        }

        if self.failed.len() > 0 {
            println!(
                "\n{}",
                format!("  Failed ({}x)", self.failed.len())
                    .bright_red()
                    .bold()
            );
            for output in self.failed.iter() {
                output.print();
            }
        }
    }
}

pub struct TestReport {
    failed: Vec<(Test, HashSet<(String, String)>)>,
}

impl TestReport {
    pub fn from_report(report: Report, tests: Vec<Test>) -> Self {
        let mut failed = Vec::<(Test, HashSet<(String, String)>)>::new();
        let group = group_outputs(report.success);
        for test in tests {
            // Get a test that maches the group with similar input
            if let (Some(test_input), Some(test_output)) = (&test.input, &test.output) {
                if let Some((_, outputs)) = group.iter().find(|(i, _)| i == test_input) {
                    // Check if the output is correct
                    let mut sets = HashSet::new();
                    for output in outputs {
                        if &output.stdout != test_output {
                            sets.insert((output.stdout.clone(), output.stderr.clone()));
                        }
                    }
                    if sets.len() > 0 {
                        failed.push((test, sets));
                    }
                }
            }
        }
        Self { failed }
    }

    pub fn print_summary(&self) {
        println!(
            "\n\n{} {} {}\n",
            ">>".dimmed(),
            "Test Summary".bold().underline(),
            "<<".dimmed(),
        );
        if self.failed.len() > 0 {
            for (test, outputs) in self.failed.iter() {
                println!("{}", format_args!("  Failed {}", test.name).red());
                if let Some(input) = &test.input {
                    println!("{}", format_args!("  Input:\n{}", input.trim_end()).cyan());
                }
                let output = test.output.as_ref().unwrap();
                for (stdout, stderr) in outputs {
                    show_diff(&stdout, &output);
                    if stderr.len() > 0 {
                        println!("stderr: {}", stderr);
                    }
                }
            }
        } else {
            println!("{}", "Passed all tests!".green())
        }
    }
}

fn group_outputs(outputs: Vec<ProcessOutput>) -> Vec<(String, Vec<ProcessOutput>)> {
    let mut groups = HashMap::<String, Vec<ProcessOutput>>::new();
    for output in outputs.into_iter() {
        if let Some(input) = &output.stdin {
            if let Some(val) = groups.get_mut(input) {
                val.push(output);
            } else {
                groups.insert(input.clone(), vec![output]);
            }
        }
    }
    groups.into_iter().map(|(i, o)| (i, o)).collect()
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
