//! nali-rs: Rust implementation of IP geolocation lookup tool
//!
//! A high-performance Rust version of the original nali tool for querying
//! IP geographic information and CDN providers offline.

// Public modules
pub mod config;
pub mod database;
pub mod error;
pub mod entity;
pub mod regex;
pub mod utils;
pub mod download;
pub mod cli;

// Re-export commonly used types
pub use config::{AppConfig, DatabaseConfig, OutputConfig, GlobalConfig, DatabaseInfo};
pub use database::{Database, DatabaseType, GeoLocation, CdnProvider, DatabaseManager};
pub use error::{NaliError, Result};
pub use entity::{Entity, EntityType, Entities};
