use crate::{
    error::Error,
    runner::{Controller, RunCommand},
};
use rand::{seq::IteratorRandom, Rng};
use std::{str::FromStr, time::Duration};

#[derive(Debug, Clone)]
pub enum InputTemplate {
    List(Vec<Self>),
    Set(Vec<Self>),
    Const(String),
    RandInt { min: i64, max: i64 },
}

impl InputTemplate {
    pub fn generate(&self) -> Option<String> {
        let mut rng = rand::thread_rng();
        match self {
            InputTemplate::List(items) => Some(
                items
                    .into_iter()
                    .map(|i| i.generate())
                    .flatten()
                    .collect::<Vec<String>>()
                    .join(""),
            ),
            InputTemplate::Set(items) => items
                .iter()
                .choose_stable(&mut rng)
                .map(|i| i.generate())
                .flatten(),
            InputTemplate::Const(s) => Some(s.clone()),
            InputTemplate::RandInt { min, max } => Some(rng.gen_range(*min..=*max).to_string()),
        }
    }
}

const OPEN_CHARS: [char; 3] = ['[', '{', '('];
const CLOSE_CHARS: [char; 3] = [']', '}', ')'];
const SEP: char = ',';

fn split_parts(s: &str) -> Vec<&str> {
    let mut depth = 0;
    let mut parts = Vec::new();
    let mut start = 0;
    for (i, c) in s.char_indices() {
        if OPEN_CHARS.contains(&c) {
            depth += 1;
        } else if CLOSE_CHARS.contains(&c) {
            depth -= 1;
        } else if depth == 0 && c == SEP {
            parts.push(&s[start..i]);
            start = i + 1;
        }
    }
    parts.push(&s[start..s.len()]);
    return parts;
}

impl FromStr for InputTemplate {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let part = s.trim();
        if part.starts_with('[') && part.ends_with(']') {
            let mut subs = Vec::new();
            for sub in split_parts(&part[1..part.len() - 1]) {
                subs.push(Self::from_str(sub)?)
            }
            Ok(Self::List(subs))
        } else if part.starts_with('{') && part.ends_with('}') {
            let mut subs = Vec::new();
            for sub in split_parts(&part[1..part.len() - 1]) {
                subs.push(Self::from_str(sub)?)
            }
            Ok(Self::Set(subs))
        } else if part.starts_with('(') && part.ends_with(')') {
            let spilts: Vec<&str> = part[1..part.len() - 1]
                .split('-')
                .map(|s| s)
                .collect();
            if spilts.len() != 2 {
                return Err(Error::Prase("Invalid count of '-'.".to_string()));
            }
            let min = spilts[0].parse::<i64>().map_err(|e| {
                Error::Prase(format!("Invalid left side '{}' of range: {e}", spilts[0]))
            })?;
            let max = spilts[1].parse::<i64>().map_err(|e| {
                Error::Prase(format!("Invalid right side '{}' of range: {e}", spilts[1]))
            })?;
            Ok(Self::RandInt { min, max })
        } else {
            Ok(Self::Const(s.to_string()))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fuzzer {
    template: InputTemplate,
    command: String,
    time_limit: Option<Duration>,
}

impl Fuzzer {
    pub fn new(template: InputTemplate, command: String, time_limit: Option<Duration>) -> Self {
        Self {
            template,
            command,
            time_limit,
        }
    }
}

impl Controller for Fuzzer {
    fn get(&mut self) -> Option<RunCommand> {
        Some(RunCommand {
            command: &self.command,
            input: self.template.generate().map(|i| format!("{i}\n")),
            time_limit: self.time_limit,
        })
    }
}
