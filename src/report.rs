use owo_colors::OwoColorize;

use crate::{loader::TestCase, tester::TestError};

pub struct TestReport {
    pub timed_out: Vec<TestCase>,
    pub failed: Vec<(TestCase, TestError)>,
    pub passed: Vec<TestCase>,
}

impl TestReport {
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

        for (test, error) in &self.failed {
            match error {
                TestError::Killed { status } => {
                    println!("Test {} was killed with status {}", test.name, status);
                }
                TestError::Fail { expected, actual } => {
                    println!(
                        "Test '{}' failed, expected: {}, got: {}",
                        test.name,
                        String::from_utf8_lossy(expected)
                            .trim_end_matches('\n')
                            .green(),
                        String::from_utf8_lossy(actual).trim_end_matches('\n').red()
                    );
                }
            }
        }
    }
}
