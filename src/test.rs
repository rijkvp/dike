use crate::{
    runner::{Controller, RunCommand},
    testfile::TestFile,
};
use rand::seq::IteratorRandom;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Test {
    pub name: String,
    pub input: Option<String>,
    pub output: Option<String>,
}

impl Test {
    pub fn new(name: String, input: Option<String>, output: Option<String>) -> Self {
        Self {
            name,
            input,
            output,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tester {
    tests: HashMap<u64, Test>,
    command: String,
    left: Vec<u64>,
    time_limit: Duration,
}

impl Tester {
    pub fn new(testfile: &TestFile, command: String, time_limit: Duration) -> Self {
        let tests: HashMap<u64, Test> = testfile
            .tests
            .iter()
            .enumerate()
            .map(|(n, t)| (n as u64, t.clone()))
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
        let index = self
            .left
            .iter()
            .choose_stable(&mut rand::thread_rng())?
            .clone();
        self.left.retain(|i| i != &index);
        Some(RunCommand {
            command: self.command.clone(),
            input: self.tests[&index].input.clone(),
            time_limit: Some(self.time_limit),
        })
    }
}
