//! Path utilities for configuration and database files
//!
//! Handles XDG directory specifications and path resolution.

use crate::error::{NaliError, Result};
use std::path::{Path, PathBuf};
use std::env;

/// Get the nali configuration directory
///
/// Priority:
/// 1. NALI_CONFIG_HOME environment variable
/// 2. NALI_HOME environment variable
/// 3. XDG_CONFIG_HOME/nali-rs
/// 4. ~/.config/nali-rs (fallback)
pub fn config_dir() -> Result<PathBuf> {
    if let Ok(path) = env::var("NALI_CONFIG_HOME") {
        return Ok(PathBuf::from(path));
    }

    if let Ok(path) = env::var("NALI_HOME") {
        return Ok(PathBuf::from(path));
    }

    if let Some(config_dir) = dirs::config_dir() {
        return Ok(config_dir.join("nali-rs"));
    }

    Err(NaliError::config("无法确定配置目录"))
}

/// Get the nali data directory for databases
///
/// Priority:
/// 1. NALI_DB_HOME environment variable
/// 2. NALI_HOME environment variable
/// 3. XDG_DATA_HOME/nali-rs
/// 4. ~/.local/share/nali-rs (fallback)
pub fn data_dir() -> Result<PathBuf> {
    if let Ok(path) = env::var("NALI_DB_HOME") {
        return Ok(PathBuf::from(path));
    }

    if let Ok(path) = env::var("NALI_HOME") {
        return Ok(PathBuf::from(path));
    }

    if let Some(data_dir) = dirs::data_dir() {
        return Ok(data_dir.join("nali-rs"));
    }

    Err(NaliError::config("无法确定数据目录"))
}

/// Get the path to the config file
pub fn config_file() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.yaml"))
}

/// Get the path to a database file
pub fn database_file(name: &str) -> Result<PathBuf> {
    Ok(data_dir()?.join(name))
}

/// Ensure directory exists, create if necessary
pub fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| NaliError::config(format!("创建目录失败: {}", e)))?;
    }
    Ok(())
}

/// Ensure the nali directories exist
pub fn ensure_nali_dirs() -> Result<()> {
    ensure_dir(&config_dir()?)?;
    ensure_dir(&data_dir()?)?;
    Ok(())
}

/// Expand tilde (~) in path
pub fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir() {
        let dir = config_dir();
        assert!(dir.is_ok());
        let path = dir.unwrap();
        assert!(path.to_string_lossy().contains("nali-rs"));
    }

    #[test]
    fn test_data_dir() {
        let dir = data_dir();
        assert!(dir.is_ok());
        let path = dir.unwrap();
        assert!(path.to_string_lossy().contains("nali-rs"));
    }

    #[test]
    fn test_expand_tilde() {
        let path = expand_tilde("~/test");
        assert!(!path.to_string_lossy().starts_with("~"));
    }
}
