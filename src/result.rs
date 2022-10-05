use owo_colors::OwoColorize;
use std::process::Output;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum ProcessResult {
    Unfinished {
        stdin: Option<String>,
    },
    Finished {
        status: Option<i32>,
        duration: Duration,
        stdin: Option<String>,
        stdout: String,
        stderr: String,
    },
}

impl ProcessResult {
    pub fn from_output(output: &Output, stdin: Option<String>, duration: Duration) -> Self {
        Self::Finished {
            status: output.status.code(),
            duration,
            stdin,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }
}

pub fn summarize_results(results: &Vec<ProcessResult>) {
    let mut finished = results.clone();
    finished.retain(|r| match r {
        ProcessResult::Finished {..} => true,
        _ => false,
    } );
    // let mut errors = finished.clone();
    // errors.retain(|r| r.status != Some(0));
    println!(
        "\n{}: {}, {}",
        format!("{} results", results.len()).white(),
        format!("{} finished", finished.len()).green(),
        format!("{} unfinished", results.len() - finished.len()).red(),
    );
    // for error in errors.iter().take(3) {
    //     let status = error
    //         .status
    //         .map(|c| c.to_string())
    //         .unwrap_or("(??)".to_string());
    //     println!("{}", format!("Error ({})", status).red().bold());
    //     if let Some(stdin) = &error.stdin {
    //         println!("Standard Input: {}", stdin.trim());
    //     }
    //     if error.stdout.len() > 0 {
    //         println!("Standard Output: {}", error.stdout.trim());
    //     }
    //     if error.stderr.len() > 0 {
    //         println!("Standard Error: {}", error.stderr.trim());
    //     }
    // }
}
