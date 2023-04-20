use crate::runner::{run_command, CmdOptions, CmdStatus, Controller};
use std::time::Duration;
use tracing::error;

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
    fn get(&mut self) -> Option<CmdOptions> {
        let CmdStatus::Terminated(input) = run_command(CmdOptions::new(&self.input_cmd, None, self.time_limit)) else {
            error!("Input command didn't terminate");
            return None;
        };
        if input.status != Some(0) {
            error!(
                "Input command failed (exit status: {})",
                input
                    .status
                    .map(|s| s.to_string())
                    .unwrap_or("??".to_string())
            );
            return None;
        }
        Some(CmdOptions::new(
            &self.cmd,
            Some(input.stdout),
            self.time_limit,
        ))
    }
}
