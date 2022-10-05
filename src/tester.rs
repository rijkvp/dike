use crate::{
    runner::{Controller, RunCommand},
    testfile::{TestCase, TestFile},
};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Tester {
    tests: HashMap<u64, TestCase>,
    command: String,
    left: Vec<u64>,
    time_limit: Duration,
}

impl Tester {
    pub fn new(testfile: TestFile, command: String, time_limit: Duration) -> Self {
        let tests: HashMap<u64, TestCase> = testfile
            .tests
            .into_iter()
            .enumerate()
            .map(|(n, t)| (n as u64, t))
            .collect();
        let left = tests.keys().map(|n| *n).collect();
        Self {
            tests,
            command,
            left,
            time_limit,
        }
    }
}

impl Controller for Tester {
    fn get(&mut self) -> Option<RunCommand> {
        self.left.pop().map(|i| {
            let test = &self.tests[&i];
            RunCommand {
                command: self.command.clone(),
                input: test.input.clone(),
                time_limit: Some(self.time_limit),
            }
        })
    }
}
