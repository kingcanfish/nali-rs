//! Database module for nali-rs
//!
//! This module provides a unified interface for various IP geolocation databases.
//! Each database type implements the common trait to provide consistent API.

use crate::error::Result;
use std::net::IpAddr;

/// Common result type for IP geolocation lookups
#[derive(Debug, Clone, serde::Serialize)]
pub struct GeoLocation {
    pub ip: IpAddr,
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub isp: Option<String>,
    pub country_code: Option<String>,
    pub timezone: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

/// CDN provider information
#[derive(Debug, Clone, serde::Serialize)]
pub struct CdnProvider {
    pub domain: String,
    pub provider: String,
    pub description: Option<String>,
}

/// Database type enumeration
#[derive(Debug, Clone)]
pub enum DatabaseType {
    QQwry,       // 纯真IPv4数据库
    ZXIPv6Wry,   // ZX IPv6数据库
    GeoIP2,      // GeoIP2数据库
    IPIP,        // IPIP数据库
    IP2Region,   // ip2region数据库
    DBIP,        // DB-IP数据库
    IP2Location, // IP2Location数据库
    CDN,         // CDN数据库
}

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

/// Factory for creating database instances
pub struct DatabaseFactory;

impl DatabaseFactory {
    pub fn create(db_type: DatabaseType) -> Box<dyn Database + Send + Sync> {
        match db_type {
            DatabaseType::QQwry => Box::new(QQwryDatabase::new()),
            DatabaseType::ZXIPv6Wry => Box::new(ZXIPv6Database::new()), // 使用真实实现
            DatabaseType::GeoIP2 => Box::new(GeoIP2Database::new()),
            DatabaseType::IPIP => Box::new(IPIPDatabase::new()),
            DatabaseType::IP2Region => Box::new(IP2RegionDatabase::new()),
            DatabaseType::DBIP => Box::new(DBIPDatabase::new()),
            DatabaseType::IP2Location => Box::new(IP2LocationDatabase::new()),
            DatabaseType::CDN => Box::new(CDNDatabase::new()),
        }
    }
}

// Database implementations modules
pub mod common;
pub mod dbip;
pub mod geoip2;
pub mod ip2location;
pub mod ip2region;
pub mod ipip;
pub mod qqwry;
pub mod zxipv6;
pub mod manager;

// Re-export database implementations
pub use common::CDNDatabase;
pub use dbip::DBIPDatabase;
pub use geoip2::GeoIP2Database;
pub use ip2location::IP2LocationDatabase;
pub use ip2region::IP2RegionDatabase;
pub use ipip::IPIPDatabase;
pub use qqwry::QQwryDatabase;
pub use zxipv6::ZXIPv6Database;
pub use manager::DatabaseManager;
