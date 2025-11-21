//! Trait definitions for the database module
//!
//! This module defines the common interface that all database implementations must follow.

use crate::error::Result;
use std::net::IpAddr;

use super::types::{CdnProvider, DatabaseType, GeoLocation};

/// Common trait for all database implementations
pub trait Database {
    fn name(&self) -> &str;
    fn database_type(&self) -> DatabaseType;
    fn supports_ipv4(&self) -> bool;
    fn supports_ipv6(&self) -> bool;
    fn supports_cdn(&self) -> bool;

    /// Look up IP geolocation information
    fn lookup_ip(&self, ip: IpAddr) -> Result<Option<GeoLocation>>;

    /// Look up CDN provider information
    fn lookup_cdn(&self, domain: &str) -> Result<Option<CdnProvider>>;

    /// Check if database is loaded and ready to use
    fn is_loaded(&self) -> bool;

    /// Load database from file
    fn load_from_file(&mut self, file_path: &str) -> Result<()>;
}
