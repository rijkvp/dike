use std::str::FromStr;

use crate::{error::Error, result::ProcessResult};

#[derive(Debug, Clone)]
pub struct TestFile {
    pub tests: Vec<TestCase>,
}

impl TestFile {
    pub fn from_results(results: &Vec<ProcessResult>) -> TestFile {
        let mut tests = Vec::new();
        for (n, result) in results.into_iter().enumerate() {
            tests.push(TestCase::new(format!("Test #{}", n), result.stdin.clone(), Some(result.stdout.clone())));
        }
        Self { tests }
    }
}

impl FromStr for TestFile {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tests = Vec::new();
        for fragment in s.split('|') {
            let fragment = fragment.trim();
            if fragment.is_empty() {
                continue;
            }
            let mut name = String::new();
            let mut input = String::new();
            let mut string = String::new();
            let mut field = -1;
            for (n, line) in fragment.lines().enumerate() {
                if n == 0 {
                    name += line;
                }
                if line.starts_with('<') {
                    if &line[1..2] == "<" {
                        field = 1;
                    } else {
                        input.push_str(&line[1..].trim_start());
                        input.push('\n');
                        field = 0;
                    }
                } else if line.starts_with('>') {
                    if &line[1..2] == ">" {
                        field = 2;
                    } else {
                        string.push_str(&line[1..].trim_start());
                        string.push('\n');
                        field = 0;
                    }
                } else if field == 1 {
                    input.push_str(line);
                    input.push('\n');
                } else if field == 2 {
                    string.push_str(line);
                    string.push('\n');
                }
            }
            let input = {
                if !input.is_empty() {
                    Some(input)
                } else {
                    None
                }
            };
            let string = {
                if !string.is_empty() {
                    Some(string)
                } else {
                    None
                }
            };
            let test = TestCase::new(name, input, string);
            tests.push(test);
        }
        Ok(Self { tests })
    }
}

impl ToString for TestFile {
    fn to_string(&self) -> String {
        let mut string = String::new();
        for test in &self.tests {
            string.push_str("| ");
            string.push_str(&test.name);
            string.push('\n');
            if let Some(input) = &test.input {
                if input.lines().count() > 1 {
                    string.push_str("<< ");
                } else {
                    string.push_str("< ");
                }
                string.push_str(&input);
            }
            if let Some(output) = &test.output {
                if output.lines().count() > 1 {
                    string.push_str(">> ");
                } else {
                    string.push_str("> ");
                }
                string.push_str(&output);
            }
        }
        string
    }
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub input: Option<String>,
    pub output: Option<String>,
}

impl TestCase {
    pub fn new(name: String, input: Option<String>, output: Option<String>) -> Self {
        Self {
            name,
            input,
            output,
        }
    }
}
