//! nali-rs: Rust implementation of IP geolocation lookup tool
//!
//! A high-performance Rust version of the original nali tool for querying
//! IP geographic information and CDN providers offline.

// Core types and error handling
pub use anyhow::{anyhow, Context, Result};
pub use log::{debug, error, info, warn};
pub use serde::{Deserialize, Serialize};
pub use std::net::IpAddr;

// Application configuration and settings
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_config: DatabaseConfig,
    pub output_config: OutputConfig,
    pub global_config: GlobalConfig,
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub selected_ip4_db: String,
    pub selected_ip6_db: String,
    pub selected_cdn_db: String,
    pub selected_language: String,
    pub database_paths: std::collections::HashMap<String, String>,
}

/// Output configuration
#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub enable_colors: bool,
    pub output_json: bool,
    pub use_gbk: bool,
}

/// Global configuration
#[derive(Debug, Clone)]
pub struct GlobalConfig {
    pub verbose: bool,
    pub config_path: Option<String>,
    pub work_dir: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database_config: DatabaseConfig::default(),
            output_config: OutputConfig::default(),
            global_config: GlobalConfig::default(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            selected_ip4_db: "qqwry".to_string(),
            selected_ip6_db: "zxipv6wry".to_string(),
            selected_cdn_db: "cdn".to_string(),
            selected_language: "zh-CN".to_string(),
            database_paths: std::collections::HashMap::new(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            enable_colors: true,
            output_json: false,
            use_gbk: false,
        }
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            config_path: None,
            work_dir: None,
        }
    }
}
