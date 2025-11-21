//! Database downloader module
//!
//! Handles downloading and updating database files from remote sources.

use crate::config::AppConfig;
use crate::error::{NaliError, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use sevenz_rust::decompress_file;

// Constants
const DEFAULT_TIMEOUT_SECS: u64 = 300;
const DOWNLOAD_BUFFER_SIZE: usize = 8192;

/// Database downloader
///
/// Handles downloading database files from remote URLs with progress tracking,
/// automatic retries, and support for compressed archives (7z).
pub struct Downloader {
    client: reqwest::Client,
}

impl Downloader {
    /// Create a new downloader
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(format!("nali-rs/{}", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .map_err(|e| NaliError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client })
    }

    /// Download a file from URL to destination path
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to download from
    /// * `dest` - The destination file path
    /// * `show_progress` - Whether to display a progress bar
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Download completed successfully
    /// * `Err(NaliError)` - Download failed
    pub async fn download_file(&self, url: &str, dest: &Path, show_progress: bool) -> Result<()> {
        log::info!("Downloading from: {}", url);
        log::info!("Saving to: {:?}", dest);

        // Create parent directory if it doesn't exist
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(NaliError::IoError)?;
        }

        // Start download
        let response = self.client.get(url)
            .send()
            .await
            .map_err(|e| NaliError::NetworkError(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            return Err(NaliError::DownloadError(format!(
                "HTTP error: {} - {}",
                response.status(),
                url
            )));
        }

        // Get content length for progress bar
        let total_size = response.content_length();

        // Setup progress bar
        let pb = if show_progress && total_size.is_some() {
            let pb = ProgressBar::new(total_size.unwrap());
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            pb.set_message(format!("Downloading {}", url.split('/').next_back().unwrap_or("database")));
            Some(pb)
        } else {
            None
        };

        // Download and write to file
        let mut file = File::create(dest)
            .map_err(NaliError::IoError)?;

        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk
                .map_err(|e| NaliError::NetworkError(format!("Failed to read chunk: {}", e)))?;

            file.write_all(&chunk)
                .map_err(NaliError::IoError)?;

            downloaded += chunk.len() as u64;
            if let Some(ref pb) = pb {
                pb.set_position(downloaded);
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message(format!("Downloaded {}", dest.file_name().unwrap().to_string_lossy()));
        }

        log::info!("Successfully downloaded to: {:?}", dest);
        Ok(())
    }

    /// Download database by name
    pub async fn download_database(&self, config: &AppConfig, db_name: &str) -> Result<()> {
        // Find database info
        let db_info = config.database.databases.iter()
            .find(|db| db.name == db_name || db.name_alias.contains(&db_name.to_string()))
            .ok_or_else(|| NaliError::DatabaseNotFound(format!("Database not found: {}", db_name)))?;

        if db_info.download_urls.is_empty() {
            return Err(NaliError::DownloadError(format!(
                "No download URL configured for database: {}",
                db_name
            )));
        }

        // Get destination path
        let dest_path = config.get_database_path(&db_info.name)?;

        // Special handling for CDN database - download and merge from multiple sources
        if db_name == "cdn" {
            return self.download_and_merge_cdn(db_info, &dest_path).await;
        }

        // Try each download URL until one succeeds
        let mut last_error = None;
        for url in &db_info.download_urls {
            match self.try_download_and_extract(url, &dest_path, db_name).await {
                Ok(_) => {
                    println!("✓ Successfully downloaded {} database", db_info.name);
                    return Ok(());
                }
                Err(e) => {
                    log::warn!("Failed to download from {}: {}", url, e);
                    last_error = Some(e);
                }
            }
        }

        // All URLs failed
        Err(last_error.unwrap_or_else(|| {
            NaliError::DownloadError(format!("Failed to download database: {}", db_name))
        }))
    }

    /// Try to download and extract a database file from a URL
    async fn try_download_and_extract(&self, url: &str, dest_path: &Path, db_name: &str) -> Result<()> {
        // Check if URL is for a 7z file
        let is_7z = url.ends_with(".7z");

        // Download to temp file if 7z, otherwise direct to destination
        let download_path = if is_7z {
            let temp_dir = std::env::temp_dir();
            temp_dir.join(format!("{}.7z", db_name))
        } else {
            dest_path.to_path_buf()
        };

        // Download the file
        self.download_file(url, &download_path, true).await?;

        // Extract if 7z
        if is_7z {
            println!("Extracting 7z archive...");
            self.extract_7z(&download_path, dest_path, db_name).await?;
            // Clean up temp file
            let _ = std::fs::remove_file(&download_path);
        }

        Ok(())
    }

    /// Download CDN databases from multiple sources and merge them
    async fn download_and_merge_cdn(&self, db_info: &crate::config::DatabaseInfo, dest_path: &PathBuf) -> Result<()> {
        println!("Downloading CDN databases from multiple sources...");

        let mut all_cdn_data: std::collections::HashMap<String, serde_yaml::Value> = std::collections::HashMap::new();
        let mut success_count = 0;

        for (idx, url) in db_info.download_urls.iter().enumerate() {
            println!("  [{}/{}] Downloading from {}...", idx + 1, db_info.download_urls.len(), url);

            match self.download_cdn_from_url(url).await {
                Ok(cdn_data) => {
                    println!("      ✓ Downloaded {} entries", cdn_data.len());
                    // Merge data - later sources override earlier ones
                    for (key, value) in cdn_data {
                        all_cdn_data.insert(key, value);
                    }
                    success_count += 1;
                }
                Err(e) => {
                    println!("      ✗ Failed: {}", e);
                    log::warn!("Failed to download CDN data from {}: {}", url, e);
                }
            }
        }

        if success_count == 0 {
            return Err(NaliError::DownloadError(
                "Failed to download CDN database from all sources".to_string()
            ));
        }

        println!("\nMerging CDN data from {} sources...", success_count);
        println!("Total unique CDN entries: {}", all_cdn_data.len());

        // Create parent directory if needed
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(NaliError::IoError)?;
        }

        // Write merged data to file
        let yaml_content = serde_yaml::to_string(&all_cdn_data)
            .map_err(|e| NaliError::YamlError(format!("Failed to serialize CDN data: {}", e)))?;

        std::fs::write(dest_path, yaml_content)
            .map_err(NaliError::IoError)?;

        println!("✓ Successfully downloaded and merged CDN database");
        Ok(())
    }

    /// Download CDN data from a single URL
    async fn download_cdn_from_url(&self, url: &str) -> Result<std::collections::HashMap<String, serde_yaml::Value>> {
        let response = self.client.get(url)
            .send()
            .await
            .map_err(|e| NaliError::NetworkError(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            return Err(NaliError::DownloadError(format!(
                "HTTP error: {} - {}",
                response.status(),
                url
            )));
        }

        let content = response.text()
            .await
            .map_err(|e| NaliError::NetworkError(format!("Failed to read response: {}", e)))?;

        let cdn_data: std::collections::HashMap<String, serde_yaml::Value> = serde_yaml::from_str(&content)
            .map_err(|e| NaliError::YamlError(format!("Failed to parse CDN YAML: {}", e)))?;

        Ok(cdn_data)
    }

    /// Extract 7z archive
    async fn extract_7z(&self, archive_path: &Path, dest_path: &Path, db_name: &str) -> Result<()> {
        log::info!("Extracting 7z archive: {:?}", archive_path);

        // Create temp directory for extraction
        let temp_extract_dir = std::env::temp_dir().join(format!("nali-extract-{}", db_name));
        std::fs::create_dir_all(&temp_extract_dir)
            .map_err(|e| NaliError::parse(format!("Failed to create temp directory: {}", e)))?;

        // Decompress the 7z file
        decompress_file(archive_path, &temp_extract_dir)
            .map_err(|e| NaliError::parse(format!("Failed to decompress 7z: {}", e)))?;

        // Find the database file in extracted files
        // For zxipv6wry, look for ipv6wry.db
        let target_filename = match db_name {
            "zxipv6wry" | "zxipv6" => "ipv6wry.db",
            _ => return Err(NaliError::parse("Unknown 7z database type")),
        };

        // Search for the target file
        let extracted_file = find_file_recursive(&temp_extract_dir, target_filename)?;

        // Move the extracted file to destination
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(NaliError::IoError)?;
        }

        std::fs::copy(&extracted_file, dest_path)
            .map_err(NaliError::IoError)?;

        // Clean up temp directory
        let _ = std::fs::remove_dir_all(&temp_extract_dir);

        log::info!("Successfully extracted to: {:?}", dest_path);
        Ok(())
    }

    /// Download all configured databases
    pub async fn download_all(&self, config: &AppConfig) -> Result<()> {
        println!("Downloading all databases...\n");

        let mut success_count = 0;
        let mut fail_count = 0;

        for db_info in &config.database.databases {
            // Skip CDN database (it's manually created)
            if db_info.name == "cdn" {
                continue;
            }

            println!("Downloading {} database...", db_info.name);
            match self.download_database(config, &db_info.name).await {
                Ok(_) => {
                    success_count += 1;
                }
                Err(e) => {
                    eprintln!("✗ Failed to download {}: {}", db_info.name, e);
                    fail_count += 1;
                }
            }
            println!();
        }

        println!("Download complete: {} succeeded, {} failed", success_count, fail_count);

        if fail_count > 0 {
            Err(NaliError::DownloadError(format!(
                "{} databases failed to download",
                fail_count
            )))
        } else {
            Ok(())
        }
    }

    /// Update a specific database
    pub async fn update_database(&self, config: &AppConfig, db_name: &str) -> Result<()> {
        let dest_path = config.get_database_path(db_name)?;

        // Check if database exists
        let is_update = dest_path.exists();

        if is_update {
            println!("Updating {} database...", db_name);
        } else {
            println!("Installing {} database...", db_name);
        }

        self.download_database(config, db_name).await?;

        if is_update {
            println!("✓ Database updated successfully");
        } else {
            println!("✓ Database installed successfully");
        }

        Ok(())
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new().expect("Failed to create downloader")
    }
}

/// Recursively find a file by name in a directory
fn find_file_recursive(dir: &Path, filename: &str) -> Result<PathBuf> {
    for entry in std::fs::read_dir(dir).map_err(NaliError::IoError)? {
        let entry = entry.map_err(NaliError::IoError)?;
        let path = entry.path();

        if path.is_file() {
            if let Some(name) = path.file_name()
                && name == filename {
                    return Ok(path);
                }
        } else if path.is_dir()
            && let Ok(found) = find_file_recursive(&path, filename) {
                return Ok(found);
            }
    }

    Err(NaliError::parse(format!("File not found: {}", filename)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downloader_creation() {
        let downloader = Downloader::new();
        assert!(downloader.is_ok());
    }
}
