use owo_colors::OwoColorize;

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
        println!("{}", "Test summary".bold());
        println!(
            "Finished: {}/{}",
            self.terminated_count(),
            self.total_count()
        );
        println!("Passed: {}/{}", self.passed.len(), self.terminated_count());
        println!("Failed: {}/{}", self.failed.len(), self.terminated_count());

        for result in &self.failed {
            match result.result_type {
                ResultType::Pass => continue,
                ResultType::Signaled => {
                    println!(
                        "Test '{}' failed after {:?}, signaled",
                        result.testcase.name, result.duration
                    );
                }
                ResultType::WrongOutput => {
                    println!(
                        "Test '{}' failed after {:?}, expected: {}, got: {}",
                        result.testcase.name,
                        result.duration,
                        String::from_utf8_lossy(&result.testcase.output)
                            .trim_end_matches('\n')
                            .green(),
                        String::from_utf8_lossy(&result.stdout)
                            .trim_end_matches('\n')
                            .red()
                    );
                }
            }
            if let Some(status) = result.status {
                println!("Exit status: {}", status);
            } else {
                println!("Exit status: unknown");
            }
            if !result.stderr.is_empty() {
                println!(
                    "Standard error: {}",
                    String::from_utf8_lossy(&result.stderr).trim_end_matches('\n')
                );
            }
        }
    }
}
