use std::io;
use thiserror::Error;

/// Represent all LeetUp error
#[derive(Error, Debug)]
#[error("...")]
pub enum LeetUpError {
    /// Any Error
    Any(#[from] anyhow::Error),

    /// Reqwest Error
    Reqwest(#[from] reqwest::Error),

    /// IO Error
    Io(#[from] io::Error),

    /// Serde Error
    Serde(#[from] serde_json::Error),

    /// Unexpected Command Error
    #[error("Unexpected command")]
    UnexpectedCommand,
}

/// Handle Result<T, LeetUpError>
pub type Result<T> = anyhow::Result<T, LeetUpError>;
