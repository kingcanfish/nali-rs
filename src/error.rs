//! Error types for nali-rs
//!
//! This module defines custom error types using thiserror for better error handling.

use thiserror::Error;

/// Main error type for nali-rs
#[derive(Error, Debug)]
pub enum NaliError {
    /// Database not found
    #[error("数据库未找到: {0}")]
    DatabaseNotFound(String),

    /// Invalid IP address
    #[error("无效的IP地址: {0}")]
    InvalidIp(String),

    /// Invalid domain name
    #[error("无效的域名: {0}")]
    InvalidDomain(String),

    /// Database parsing error
    #[error("数据库解析错误: {0}")]
    ParseError(String),

    /// Database not loaded
    #[error("数据库未加载: {0}")]
    DatabaseNotLoaded(String),

    /// Database corrupted
    #[error("数据库已损坏: {0}")]
    DatabaseCorrupted(String),

    /// Configuration error
    #[error("配置错误: {0}")]
    ConfigError(String),

    /// Network error
    #[error("网络错误: {0}")]
    NetworkError(String),

    /// Download error
    #[error("下载失败: {0}")]
    DownloadError(String),

    /// File I/O error
    #[error("文件I/O错误: {0}")]
    IoError(#[from] std::io::Error),

    /// Encoding error
    #[error("编码转换错误: {0}")]
    EncodingError(String),

    /// Regex error
    #[error("正则表达式错误: {0}")]
    RegexError(#[from] regex::Error),

    /// YAML parsing error
    #[error("YAML解析错误: {0}")]
    YamlError(String),

    /// JSON parsing error
    #[error("JSON解析错误: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Index out of bounds
    #[error("索引越界: offset={0}, size={1}")]
    IndexOutOfBounds(usize, usize),

    /// Other error
    #[error("其他错误: {0}")]
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
