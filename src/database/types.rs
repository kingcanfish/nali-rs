//! Type definitions for the database module
//!
//! This module contains common types used across all database implementations.

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
