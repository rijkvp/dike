use owo_colors::OwoColorize;
use similar::{ChangeTag, TextDiff};

use crate::{
    loader::TestCase,
    tester::{ResultType, TestResult},
};

pub struct TestReport {
    timed_out: Vec<TestCase>,
    failed: Vec<TestResult>,
    passed: Vec<TestResult>,
}

impl TestReport {
    pub fn from_results(timed_out: Vec<TestCase>, results: Vec<TestResult>) -> Self {
        let mut failed = Vec::new();
        let mut passed = Vec::new();

        for result in results {
            match result.result_type {
                ResultType::Pass => {
                    passed.push(result);
                }
                ResultType::Signaled | ResultType::WrongOutput => {
                    failed.push(result);
                }
            }
        }

        Self {
            timed_out,
            failed,
            passed,
        }
    }

    fn terminated_count(&self) -> usize {
        self.passed.len() + self.failed.len()
    }

    fn total_count(&self) -> usize {
        self.timed_out.len() + self.terminated_count()
    }

    pub fn print_report(&self) {
        println!("{}", "== TEST SUMMARY ==".bold());
        println!(
            "{: >10} {}/{}",
            "finished:".bold(),
            self.terminated_count(),
            self.total_count()
        );
        let failed_count = self.failed.len();
        let passed_count = self.passed.len();
        println!(
            "{: >10} {}",
            "passed:".bold(),
            if passed_count == self.terminated_count() {
                format!("{0}/{0}", passed_count).bright_green().to_string()
            } else {
                format!(
                    "{}/{}",
                    passed_count.bright_yellow(),
                    self.terminated_count()
                )
            },
        );
        println!(
            "{: >10} {}",
            "failed:".bold(),
            if failed_count == 0 {
                format!("{}/{}", failed_count, self.terminated_count())
                    .bright_green()
                    .to_string()
            } else {
                format!("{}/{}", failed_count.bright_red(), self.terminated_count())
            },
        );

        for result in &self.failed {
            match result.result_type {
                ResultType::Pass => continue,
                ResultType::Signaled => {
                    print_failed(&result);
                    println!(
                        "runtime error, exit status: {}",
                        (if let Some(status) = result.status {
                            status.to_string()
                        } else {
                            "unknown".to_string()
                        })
                        .bold()
                    );
                }
                ResultType::WrongOutput => {
                    print_failed(&result);
                    println!("wrong output");
                    let output = String::from_utf8_lossy(&result.stdout);
                    let expected = String::from_utf8_lossy(&result.testcase.output);
                    print_diff(&output, &expected);
                }
            }
            if !result.stderr.is_empty() {
                let stderr = String::from_utf8_lossy(&result.stderr);
                let stderr = stderr.trim_end_matches('\n');
                if stderr.lines().count() > 1 {
                    println!("{}\n{}", "stderr:".bold(), stderr);
                } else {
                    println!("{: >10} {}", "stderr:".bold(), stderr);
                }
            }
        }
    }
}

fn print_failed(result: &TestResult) {
    print!(
        "{} {} ({}) ",
        "FAILED".bright_red().bold(),
        result.testcase.name,
        format_duration(result.duration)
    );
}

fn print_diff(output: &str, expected: &str) {
    // trailing newlines are not visible in the output
    let output = output.trim_end_matches('\n');
    let expected = expected.trim_end_matches('\n');

    if output.lines().count() == 1 && expected.lines().count() == 1 {
        println!("{: >10} {}", "got:".bold(), output);
        println!("{: >10} {}", "expected:".bold(), expected);
    } else {
        println!("{}", "output diff:".bold());
        let diff = TextDiff::from_lines(output, expected);
        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Delete => {
                    print!("{} {}", "-".red().bold(), change.dimmed().strikethrough())
                }
                ChangeTag::Insert => print!("{} {}", "+".blue().bold(), change.blue()),
                ChangeTag::Equal => print!("{} {}", " ", change.green()),
            };
        }
    }
}

fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    format!("{}.{:02}s", secs, millis)
}
