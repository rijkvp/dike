use std::{
    io::Write,
    process::{Command, Output, Stdio},
    time::{Duration, Instant},
};


#[derive(Debug, Clone)]
pub struct ProcessResult {
    pub status: Option<i32>,
    pub duration: Duration,
    pub stdin: Option<String>,
    pub stdout: String,
    pub stderr: String,
}

impl ProcessResult {
    pub fn from_output(output: &Output, stdin: Option<String>, duration: Duration) -> Self {
        Self {
            status: output.status.code(),
            duration,
            stdin,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }
}

/// Runs the command and optionally writes input to stdin.
pub fn run(cmd: &str, input: Option<String>) -> ProcessResult {
    let start_time = Instant::now();
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    if let Some(input) = &input {
        stdin
            .write_all(input.as_bytes())
            .expect("failed to write stdin");
    }
    let output = child.wait_with_output().unwrap();
    ProcessResult::from_output(&output, input, start_time.elapsed())
}

pub fn summarize_results(results: Vec<ProcessResult>) {
    let mut errors = results.clone();
    errors.retain(|r| r.status != Some(0));
    println!(
        "{} success, {} errors ",
        results.len() - errors.len(),
        errors.len()
    );
    for error in errors.iter().take(3) {
        println!(
            "{}",
            error
                .status
                .map(|c| c.to_string())
                .unwrap_or("(none)".to_string())
        );
        println!(
            "{:?}{}{}",
            error.stdin,
            format!("stdout:\n{}", error.stdout),
            format!("stderr:\n{}", error.stderr)
        );
    }
}
