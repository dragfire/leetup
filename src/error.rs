use std::io;
use thiserror::Error;

/// Represent all LeetUp error
#[derive(Error, Debug)]
#[error("...")]
pub enum LeetUpError {
    /// Any Error
    Any(#[from] anyhow::Error),

    /// IO Error
    Io(#[from] io::Error),

    /// Serde Error
    Serde(#[from] serde_json::Error),

    /// Regex Error
    Regex(#[from] regex::Error),

    /// Reqwest Error
    Reqwest(#[from] reqwest::Error),

    /// Invalid header value error
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    /// Option None Error
    #[error("Tried to unwrap None")]
    OptNone,

    /// Unexpected Command Error
    #[error("Unexpected command")]
    UnexpectedCommand,
}

/// Handle Result<T, LeetUpError>
pub type Result<T> = anyhow::Result<T, LeetUpError>;
