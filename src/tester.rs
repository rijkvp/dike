use crate::{loader::TestCase, report::TestReport, runner::ExecResult};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::time::Duration;

pub struct Tester {
    testcases: HashMap<usize, TestCase>,
    command: String,
    timeout: Option<Duration>,
    timed_out: Vec<usize>,
    failed: Vec<(usize, TestError)>,
    passed: Vec<usize>,
}

pub struct TestExec {
    pub test_id: usize,
    pub cmd: String,
    pub input: Vec<u8>,
    pub timeout: Option<Duration>,
}

pub enum TestError {
    Fail { expected: Vec<u8>, actual: Vec<u8> },
    Killed { status: i32 },
}

impl Tester {
    pub fn new(testcases: Vec<TestCase>, command: String, timeout: Option<Duration>) -> Self {
        let testcases: HashMap<usize, TestCase> = testcases
            .into_iter()
            .enumerate()
            .map(|(n, t)| (n, t))
            .collect();
        Self {
            testcases,
            command,
            timeout,
            timed_out: Vec::new(),
            failed: Vec::new(),
            passed: Vec::new(),
        }
    }

    pub fn execs(&self) -> Vec<TestExec> {
        self.testcases
            .iter()
            .map(|(id, test)| TestExec {
                test_id: *id,
                cmd: self.command.clone(),
                input: test.input.clone(),
                timeout: self.timeout,
            })
            .collect()
    }

    pub fn report(&mut self, result: ExecResult) {
        match result {
            ExecResult::Timeout { test_id } => {
                self.timed_out.push(test_id);
            }
            ExecResult::Terminated { test_id, output } => {
                if output.status != Some(0) {
                    self.failed.push((
                        test_id,
                        TestError::Killed {
                            status: output.status.unwrap(),
                        },
                    ));
                } else {
                    let expected_output = &self.testcases[&test_id].output;
                    if output.stdout == *expected_output {
                        self.passed.push(test_id);
                    } else {
                        self.failed.push((
                            test_id,
                            TestError::Fail {
                                expected: expected_output.to_vec(),
                                actual: output.stdout,
                            },
                        ));
                    }
                }
            }
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "{} / {} / {} finished",
            format!("{} passed", self.passed.len()).green().bold(),
            format!("{} failed", self.failed.len()).red().bold(),
            self.passed.len() + self.failed.len()
        )
    }

    pub fn total_count(&self) -> usize {
        self.testcases.len()
    }

    pub fn into_report(mut self) -> TestReport {
        TestReport {
            timed_out: self
                .timed_out
                .into_iter()
                .map(|idx| self.testcases.remove(&idx).unwrap())
                .collect(),
            failed: self
                .failed
                .into_iter()
                .map(|(idx, error)| (self.testcases.remove(&idx).unwrap(), error))
                .collect(),
            passed: self
                .passed
                .into_iter()
                .map(|idx| self.testcases.remove(&idx).unwrap())
                .collect(),
        }
    }
}
