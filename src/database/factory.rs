//! Factory for creating database instances
//!
//! This module provides a factory pattern implementation for creating
//! different types of database instances.

use super::traits::Database;
use super::types::DatabaseType;
use super::{
    CDNDatabase, DBIPDatabase, GeoIP2Database, IP2LocationDatabase, IP2RegionDatabase,
    IPIPDatabase, QQwryDatabase, ZXIPv6Database,
};

/// Factory for creating database instances
pub struct DatabaseFactory;

impl DatabaseFactory {
    pub fn create(db_type: DatabaseType) -> Box<dyn Database + Send + Sync> {
        match db_type {
            DatabaseType::QQwry => Box::new(QQwryDatabase::new()),
            DatabaseType::ZXIPv6Wry => Box::new(ZXIPv6Database::new()),
            DatabaseType::GeoIP2 => Box::new(GeoIP2Database::new()),
            DatabaseType::IPIP => Box::new(IPIPDatabase::new()),
            DatabaseType::IP2Region => Box::new(IP2RegionDatabase::new()),
            DatabaseType::DBIP => Box::new(DBIPDatabase::new()),
            DatabaseType::IP2Location => Box::new(IP2LocationDatabase::new()),
            DatabaseType::CDN => Box::new(CDNDatabase::new()),
        }
    }
}
