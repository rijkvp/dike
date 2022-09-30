use crate::{
    error::Error,
    runner::{Controller, RunCommand},
};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Fuzzer {
    pub command: String,
    pub min: i64,
    pub max: i64,
}

impl Fuzzer {
    pub fn parse(string: &str, command: String) -> Result<Self, Error> {
        let spilts: Vec<&str> = string.split('-').collect();
        if spilts.len() != 2 {
            return Err(Error::Prase("Invalid count of '-'.".to_string()));
        }
        let min = spilts[0]
            .trim()
            .parse::<i64>()
            .map_err(|e| Error::Prase(format!("Failed to convert first part to string: {e}")))?;
        let max = spilts[1]
            .trim()
            .parse::<i64>()
            .map_err(|e| Error::Prase(format!("Failed to convert second part to string: {e}")))?;
        Ok(Self { command, min, max })
    }
}

impl Controller for Fuzzer {
    fn get(&mut self) -> Option<RunCommand> {
        Some(RunCommand {
            command: self.command.clone(),
            input: Some(format!(
                "{}\n",
                rand::thread_rng().gen_range(self.min..self.max)
            )),
        })
    }
}
