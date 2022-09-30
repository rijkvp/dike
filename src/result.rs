use owo_colors::OwoColorize;
use std::process::Output;
use std::time::Duration;

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

pub fn summarize_results(results: Vec<ProcessResult>) {
    let mut errors = results.clone();
    errors.retain(|r| r.status != Some(0));
    println!(
        "{}: {}, {}",
        format!("{} results", results.len()).white(),
        format!("{} succes", results.len() - errors.len()).green(),
        format!("{} error", errors.len()).red(),
    );
    for error in errors.iter().take(3) {
        let status = error
            .status
            .map(|c| c.to_string())
            .unwrap_or("(??)".to_string());
        println!("{}", format!("Error ({})", status).red().bold());
        if let Some(stdin) = &error.stdin {
            println!("Standard Input: {}", stdin.trim());
        }
        if error.stdout.len() > 0 {
            println!("Standard Output: {}", error.stdout.trim());
        }
        if error.stderr.len() > 0 {
            println!("Standard Error: {}", error.stderr.trim());
        }
    }
}
