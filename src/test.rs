use crate::{
    runner::{CmdOptions, Controller},
    testfile::TestFile,
};
use rand::seq::IteratorRandom;
use std::collections::HashMap;
use std::time::Duration;
use tracing::error;

#[derive(Debug, Clone)]
pub struct Tester {
    tests: HashMap<usize, TestFile>,
    command: String,
    left: Vec<usize>,
    time_limit: Duration,
}

impl Tester {
    pub fn new(tests: Vec<TestFile>, command: String, time_limit: Duration) -> Self {
        let tests: HashMap<usize, TestFile> =
            tests.into_iter().enumerate().map(|(n, t)| (n, t)).collect();
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
    fn get(&mut self) -> Option<CmdOptions> {
        let index = self
            .left
            .iter()
            .choose_stable(&mut rand::thread_rng())?
            .clone();
        self.left.retain(|i| i != &index);
        let Ok(input) = self.tests[&index].get_input() else {
            error!("Failed to get input");
            return None;
        };
        Some(CmdOptions::new(
            &self.command,
            Some(input),
            Some(self.time_limit),
        ))
    }
}
