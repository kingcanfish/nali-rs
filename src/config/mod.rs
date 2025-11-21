//! Configuration module for nali-rs
//!
//! Handles loading and managing configuration from YAML files and environment variables.

use crate::error::{NaliError, Result};
use crate::utils::path;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub output: OutputConfig,
    pub global: GlobalConfig,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Selected IPv4 database name
    #[serde(default = "default_ipv4_database_name")]
    pub ipv4_database: String,

    /// Selected IPv6 database name
    #[serde(default = "default_ipv6_database_name")]
    pub ipv6_database: String,

    /// Selected CDN database name
    #[serde(default = "default_cdn_database_name")]
    pub cdn_database: String,

    /// Output language
    #[serde(default = "default_language")]
    pub language: String,

    /// Database file paths (name -> path)
    #[serde(default)]
    pub database_paths: HashMap<String, String>,

    /// Database list configuration
    #[serde(default)]
    pub databases: Vec<DatabaseInfo>,
}

/// Individual database information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub name: String,
    #[serde(default)]
    pub name_alias: Vec<String>,
    pub format: String,
    pub file: String,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub types: Vec<String>,
    #[serde(default)]
    pub download_urls: Vec<String>,
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Enable colored output
    #[serde(default = "default_true")]
    pub enable_colors: bool,

    /// Output in JSON format
    #[serde(default)]
    pub json: bool,

    /// Use GBK encoding for input
    #[serde(default)]
    pub use_gbk: bool,
}

/// Global configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Verbose logging
    #[serde(default)]
    pub verbose: bool,

    /// Custom config path
    #[serde(skip)]
    pub config_path: Option<PathBuf>,

    /// Custom work directory
    #[serde(skip)]
    pub work_dir: Option<PathBuf>,
}

// Default value functions
fn default_ipv4_database_name() -> String {
    env::var("NALI_DB_IP4").unwrap_or_else(|_| "qqwry".to_string())
}

fn default_ipv6_database_name() -> String {
    env::var("NALI_DB_IP6").unwrap_or_else(|_| "zxipv6wry".to_string())
}

fn default_cdn_database_name() -> String {
    env::var("NALI_DB_CDN").unwrap_or_else(|_| "cdn".to_string())
}

fn default_language() -> String {
    env::var("NALI_LANG").unwrap_or_else(|_| "zh-CN".to_string())
}

fn default_true() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            output: OutputConfig::default(),
            global: GlobalConfig::default(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            ipv4_database: default_ipv4_database_name(),
            ipv6_database: default_ipv6_database_name(),
            cdn_database: default_cdn_database_name(),
            language: default_language(),
            database_paths: HashMap::new(),
            databases: default_databases(),
        }
    }
}

fn default_databases() -> Vec<DatabaseInfo> {
    vec![
        DatabaseInfo {
            name: "qqwry".to_string(),
            name_alias: vec!["chunzhen".to_string()],
            format: "qqwry".to_string(),
            file: "qqwry.dat".to_string(),
            languages: vec!["zh-CN".to_string()],
            types: vec!["IPv4".to_string()],
            download_urls: vec![
                "https://github.com/metowolf/qqwry.dat/releases/latest/download/qqwry.dat"
                    .to_string(),
            ],
        },
        DatabaseInfo {
            name: "zxipv6wry".to_string(),
            name_alias: vec!["zxipv6".to_string()],
            format: "ipdb".to_string(),
            file: "zxipv6wry.db".to_string(),
            languages: vec!["zh-CN".to_string()],
            types: vec!["IPv6".to_string()],
            download_urls: vec!["https://ip.zxinc.org/ip.7z".to_string()],
        },
        DatabaseInfo {
            name: "cdn".to_string(),
            name_alias: vec![],
            format: "yaml".to_string(),
            file: "cdn.yml".to_string(),
            languages: vec!["zh-CN".to_string()],
            types: vec!["CDN".to_string()],
            download_urls: vec![
                "https://cdn.jsdelivr.net/gh/4ft35t/cdn/src/cdn.yml".to_string(),
                "https://raw.githubusercontent.com/4ft35t/cdn/master/src/cdn.yml".to_string(),
                "https://raw.githubusercontent.com/SukkaLab/cdn/master/src/cdn.yml".to_string(),
            ],
        },
    ]
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            enable_colors: true,
            json: false,
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

impl AppConfig {
    /// Load configuration from file and environment variables
    pub fn load() -> Result<Self> {
        // Ensure directories exist
        path::ensure_nali_dirs()?;

        let config_file = path::config_file()?;

        // Try to load from file
        let mut config = if config_file.exists() {
            let content = fs::read_to_string(&config_file)
                .map_err(|e| NaliError::config(format!("Failed to read config file: {}", e)))?;

            serde_yaml::from_str(&content)
                .map_err(|e| NaliError::YamlError(format!("Failed to parse config file: {}", e)))?
        } else {
            // Create default config
            let config = Self::default();
            config.save(&config_file)?;
            config
        };

        // Override with environment variables
        config.apply_env();

        Ok(config)
    }

    /// Apply environment variable overrides
    fn apply_env(&mut self) {
        if let Ok(val) = env::var("NALI_DB_IP4") {
            self.database.ipv4_database = val;
        }
        if let Ok(val) = env::var("NALI_DB_IP6") {
            self.database.ipv6_database = val;
        }
        if let Ok(val) = env::var("NALI_DB_CDN") {
            self.database.cdn_database = val;
        }
        if let Ok(val) = env::var("NALI_LANG") {
            self.database.language = val;
        }
    }

    /// Save configuration to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| NaliError::YamlError(format!("序列化配置失败: {}", e)))?;

        fs::write(path, yaml).map_err(|e| NaliError::config(format!("写入配置文件失败: {}", e)))?;

        Ok(())
    }

    /// Get database file path by name
    pub fn get_database_path(&self, name: &str) -> Result<PathBuf> {
        // Check if custom path is configured
        if let Some(custom_path) = self.database.database_paths.get(name) {
            return Ok(path::expand_tilde(custom_path));
        }

        // Look up in database list
        for db in &self.database.databases {
            if db.name == name || db.name_alias.contains(&name.to_string()) {
                return path::database_file(&db.file);
            }
        }

        // Default: use name as filename
        path::database_file(&format!("{}.dat", name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.database.ipv4_database, "qqwry");
        assert_eq!(config.database.ipv6_database, "zxipv6wry");
        assert!(config.output.enable_colors);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("database"));
        assert!(yaml.contains("output"));
    }
}
