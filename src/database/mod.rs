//! Database module for nali-rs
//!
//! This module provides a unified interface for various IP geolocation databases.
//! Each database type implements the common trait to provide consistent API.
//!
//! # Module Organization
//!
//! - `types`: Common type definitions (GeoLocation, CdnProvider, DatabaseType)
//! - `traits`: Trait definitions (Database trait)
//! - `factory`: Factory pattern for creating database instances
//! - `manager`: Database manager for handling multiple databases
//! - Database implementations: qqwry, zxipv6, geoip2, ipip, etc.

// Core modules
pub mod types;
pub mod traits;
pub mod factory;
pub mod manager;

// Database implementation modules
pub mod common;
pub mod dbip;
pub mod geoip2;
pub mod ip2location;
pub mod ip2region;
pub mod ipip;
pub mod qqwry;
pub mod zxipv6;

// Re-export core types and traits for convenience
pub use types::{CdnProvider, DatabaseType, GeoLocation};
pub use traits::Database;
pub use factory::DatabaseFactory;
pub use manager::DatabaseManager;

// Re-export database implementations
pub use common::CDNDatabase;
pub use dbip::DBIPDatabase;
pub use geoip2::GeoIP2Database;
pub use ip2location::IP2LocationDatabase;
pub use ip2region::IP2RegionDatabase;
pub use ipip::IPIPDatabase;
pub use qqwry::QQwryDatabase;
pub use zxipv6::ZXIPv6Database;
