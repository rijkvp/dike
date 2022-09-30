use std::str::FromStr;

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct TestFile {
    pub tests: Vec<TestCase>,
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
            let mut output = String::new();
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
                        output.push_str(&line[1..].trim_start());
                        output.push('\n');
                        field = 0;
                    }
                } else if field == 1 {
                    input.push_str(line);
                    input.push('\n');
                } else if field == 2 {
                    output.push_str(line);
                    output.push('\n');
                }
            }
            let input = {
                if !input.is_empty() {
                    Some(input)
                } else {
                    None
                }
            };
            let output = {
                if !output.is_empty() {
                    Some(output)
                } else {
                    None
                }
            };
            let test = TestCase::new(name, input, output);
            tests.push(test);
        }
        Ok(Self { tests })
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
