//! Error types for nali-rs
//!
//! This module defines custom error types using thiserror for better error handling.

use thiserror::Error;

/// Main error type for nali-rs
#[derive(Error, Debug)]
pub enum NaliError {
    /// Database not found
    #[error("Database not found: {0}")]
    DatabaseNotFound(String),

    /// Invalid IP address
    #[error("Invalid IP address: {0}")]
    InvalidIp(String),

    /// Invalid domain name
    #[error("Invalid domain: {0}")]
    InvalidDomain(String),

    /// Database parsing error
    #[error("Database parse error: {0}")]
    ParseError(String),

    /// Database not loaded
    #[error("Database not loaded: {0}")]
    DatabaseNotLoaded(String),

    /// Database corrupted
    #[error("Database corrupted: {0}")]
    DatabaseCorrupted(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Download error
    #[error("Download failed: {0}")]
    DownloadError(String),

    /// File I/O error
    #[error("File I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Encoding error
    #[error("Encoding conversion error: {0}")]
    EncodingError(String),

    /// Regex error
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    /// YAML parsing error
    #[error("YAML parse error: {0}")]
    YamlError(String),

    /// JSON parsing error
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Index out of bounds
    #[error("Index out of bounds: offset={0}, size={1}")]
    IndexOutOfBounds(usize, usize),

    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type alias for nali-rs
pub type Result<T> = std::result::Result<T, NaliError>;

impl NaliError {
    /// Create a parse error
    pub fn parse<S: Into<String>>(msg: S) -> Self {
        NaliError::ParseError(msg.into())
    }

    /// Create a config error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        NaliError::ConfigError(msg.into())
    }

    /// Create a network error
    pub fn network<S: Into<String>>(msg: S) -> Self {
        NaliError::NetworkError(msg.into())
    }

    /// Create an encoding error
    pub fn encoding<S: Into<String>>(msg: S) -> Self {
        NaliError::EncodingError(msg.into())
    }
}

/// Convert from anyhow::Error
impl From<anyhow::Error> for NaliError {
    fn from(err: anyhow::Error) -> Self {
        NaliError::Other(err.to_string())
    }
}

/// Convert from TryFromSliceError
impl From<std::array::TryFromSliceError> for NaliError {
    fn from(err: std::array::TryFromSliceError) -> Self {
        NaliError::ParseError(format!("Failed to convert byte slice: {}", err))
    }
}
