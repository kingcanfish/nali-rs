//! Main entry point for nali-rs CLI tool
//!
//! This is the Rust implementation of the nali IP geolocation lookup tool.
//! It provides the same functionality as the Go version but with better performance.

use clap::Parser;
use log::info;

mod cli;
mod config;
mod database;
mod download;
mod error;
mod regex;
mod utils;
mod entity;

// Re-export common types
pub use error::{NaliError, Result};

// Re-export database types for use in benchmarks and tests
pub use database::{
    Database, DatabaseType, GeoLocation, CdnProvider,
    QQwryDatabase, ZXIPv6Database, GeoIP2Database, IPIPDatabase,
    CDNDatabase, DBIPDatabase, IP2RegionDatabase, IP2LocationDatabase
};

use config::AppConfig;
use cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Parse command line arguments
    let cli = Cli::parse();

    info!("Starting nali-rs v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = AppConfig::load().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load config: {}, using defaults", e);
        AppConfig::default()
    });

    // Execute CLI logic
    cli.run(config).await?;

    Ok(())
}