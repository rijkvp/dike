use crate::runner::{Controller, RunCommand};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Fuzzer {
    input_cmd: String,
    cmd: String,
    time_limit: Option<Duration>,
}

impl Fuzzer {
    pub fn new(input_cmd: String, cmd: String, time_limit: Option<Duration>) -> Self {
        Self {
            input_cmd,
            cmd,
            time_limit,
        }
    }
}

impl Controller for Fuzzer {
    fn get(&mut self) -> Option<RunCommand> {
        Some(RunCommand {
            command: &self.cmd,
            input: todo!("run input command to get input"),
            time_limit: self.time_limit,
        })
    }
}
