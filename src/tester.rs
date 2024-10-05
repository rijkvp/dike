use crate::runner::CmdOutput;
use crate::{loader::TestCase, report::TestReport, runner::ExecResult};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::time::Duration;

pub struct Tester {
    testcases: HashMap<usize, TestCase>,
    cmd_args: Vec<String>,
    timeout: Option<Duration>,
    results: Vec<TestResult>,
    timed_out: Vec<usize>,
    failed: usize,
    passed: usize,
}

pub struct TestExec {
    pub test_id: usize,
    pub cmd_args: Vec<String>,
    pub input: Vec<u8>,
    pub timeout: Option<Duration>,
}

pub struct TestResult {
    pub result_type: ResultType,
    pub testcase: TestCase,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub status: Option<i32>,
    pub duration: Duration,
}

pub enum ResultType {
    Pass,
    Signaled,
    WrongOutput,
}

impl TestResult {
    fn from_output(testcase: TestCase, cmd_output: CmdOutput) -> Self {
        Self {
            result_type: if cmd_output.status == Some(0) {
                if cmd_output.stdout == testcase.output {
                    ResultType::Pass
                } else {
                    ResultType::WrongOutput
                }
            } else {
                ResultType::Signaled
            },
            testcase,
            stdout: cmd_output.stdout,
            stderr: cmd_output.stderr,
            duration: cmd_output.duration,
            status: cmd_output.status,
        }
    }
}

impl Tester {
    pub fn new(testcases: Vec<TestCase>, cmd_args: Vec<String>, timeout: Option<Duration>) -> Self {
        let testcases: HashMap<usize, TestCase> = testcases
            .into_iter()
            .enumerate()
            .map(|(n, t)| (n, t))
            .collect();
        Self {
            testcases,
            cmd_args,
            timeout,
            results: Vec::new(),
            timed_out: Vec::new(),
            failed: 0,
            passed: 0,
        }
    }

    pub fn execs(&self) -> Vec<TestExec> {
        self.testcases
            .iter()
            .map(|(id, test)| TestExec {
                test_id: *id,
                cmd_args: self.cmd_args.clone(),
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
                let testcase = self.testcases.remove(&test_id).unwrap();
                let result = TestResult::from_output(testcase, output);
                match result.result_type {
                    ResultType::Pass => {
                        self.passed += 1;
                    }
                    ResultType::Signaled | ResultType::WrongOutput { .. } => {
                        self.failed += 1;
                    }
                }
                self.results.push(result);
            }
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "{} / {} / {} finished",
            format!("{} passed", self.passed).green(),
            format!("{} failed", self.failed).red(),
            self.passed + self.failed
        )
    }

    pub fn total_count(&self) -> usize {
        self.testcases.len()
    }

    pub fn into_report(mut self) -> TestReport {
        TestReport::from_results(
            self.timed_out
                .into_iter()
                .map(|id| self.testcases.remove(&id).unwrap())
                .collect(),
            self.results,
        )
    }
}
