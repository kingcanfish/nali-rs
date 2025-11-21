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
    QQwry,       // QQwry IPv4 database
    ZXIPv6Wry,   // ZX IPv6 database
    GeoIP2,      // GeoIP2 database
    IPIP,        // IPIP database
    IP2Region,   // ip2region database
    DBIP,        // DB-IP database
    IP2Location, // IP2Location database
    CDN,         // CDN database
}
