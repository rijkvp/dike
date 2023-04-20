use crate::{
    error::Error,
    runner::{CmdOutput, CmdStatus},
    testfile::TestFile,
};
use owo_colors::OwoColorize;
use similar::{ChangeTag, TextDiff};
use std::collections::{HashMap, HashSet};

pub struct Report {
    unfinished: HashMap<String, u64>,
    failed: Vec<CmdOutput>,
    success: Vec<CmdOutput>,
}

impl Report {
    pub fn new() -> Self {
        Self {
            unfinished: HashMap::new(),
            failed: Vec::new(),
            success: Vec::new(),
        }
    }

    pub fn insert(&mut self, result: CmdStatus) {
        match result {
            CmdStatus::Killed(stdin) => {
                if let Some(stdin) = stdin {
                    if let Some(val) = self.unfinished.get_mut(&stdin) {
                        *val += 1;
                    } else {
                        self.unfinished.insert(stdin.clone(), 1);
                    }
                }
            }
            CmdStatus::Terminated(output) => {
                if output.status != Some(0) {
                    self.failed.push(output);
                } else {
                    self.success.push(output);
                }
            }
        }
    }

    fn terminated_count(&self) -> usize {
        self.success.len() + self.failed.len()
    }

    fn total_count(&self) -> usize {
        self.unfinished.len() + self.terminated_count()
    }

    pub fn summary(&self) -> String {
        format!(
            "{}/{}/{}",
            self.success.len().green().bold(),
            self.terminated_count().blue().bold(),
            self.total_count().bold(),
        )
    }

    pub fn generate(&self) {
        println!(
            "\n\n{} {} {}",
            ">>".dimmed(),
            "Summary".bold().underline(),
            "<<".dimmed(),
        );

        print!("{} total, ", self.total_count());
        if self.terminated_count() < self.total_count() {
            let terminated_percent =
                self.terminated_count() as f64 / self.terminated_count() as f64 * 100.0;
            print!(
                "{}/{} terminated ({:.2}%), ",
                self.terminated_count().red(),
                self.total_count(),
                terminated_percent,
            );
        } else {
            print!("{}, ", "all finished".green());
        }
        if self.failed.len() > 0 {
            let failed_percent = self.failed.len() as f64 / self.terminated_count() as f64 * 100.0;
            print!(
                "{}/{} failed ({:.2}%)",
                self.failed.len(),
                self.terminated_count(),
                failed_percent
            );
        } else {
            print!("{}", "all succeeded".green());
        }
        print!("\n");

        if self.unfinished.len() > 0 {
            println!(
                "\n{}",
                format!("神 Unfinished ({}x)", self.unfinished.len())
                    .yellow()
                    .bold()
            );
            for (input, count) in self.unfinished.iter() {
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
    failed: Vec<(TestFile, HashSet<(String, String)>)>,
}

impl TestReport {
    pub fn from_report(report: Report, tests: Vec<TestFile>) -> Self {
        let mut failed = Vec::<(TestFile, HashSet<(String, String)>)>::new();
        let group = group_outputs(report.success);
        for test in tests {
            // Get a test that maches the group with similar input
            if let (Ok(test_input), Ok(test_output)) = (&test.get_input(), &test.get_output()) {
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

    pub fn generate(&self) -> Result<(), Error> {
        println!(
            "\n\n{} {} {}\n",
            ">>".dimmed(),
            "Test Summary".bold().underline(),
            "<<".dimmed(),
        );
        if self.failed.len() > 0 {
            for (test, outputs) in self.failed.iter() {
                println!("{}", format_args!("  Failed test '{}'", test.name()).red());
                let input = test.get_input()?;
                let output = test.get_output()?;
                println!("{}", format_args!("  Input:\n{}", input.trim_end()).cyan());
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
        Ok(())
    }
}

fn group_outputs(outputs: Vec<CmdOutput>) -> Vec<(String, Vec<CmdOutput>)> {
    let mut groups = HashMap::<String, Vec<CmdOutput>>::new();
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
