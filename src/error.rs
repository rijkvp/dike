use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O: {0}")]
    IO(#[from] std::io::Error),
    #[error("Invalid glob pattern: {0}")]
    Glob(#[from] glob::PatternError),
}
