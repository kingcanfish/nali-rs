//! Database manager - manages database instances and caching

use crate::config::AppConfig;
use crate::database::{CdnProvider, Database, DatabaseFactory, DatabaseType, GeoLocation};
use crate::download::Downloader;
use crate::error::{NaliError, Result};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, RwLock};

/// Database manager handles loading and caching of databases
///
/// The manager provides a unified interface for querying IP geolocation and CDN
/// information. It automatically loads databases on first use and caches both
/// database instances and query results for improved performance.
///
/// # Thread Safety
///
/// DatabaseManager is thread-safe and can be shared across threads using Arc.
pub struct DatabaseManager {
    config: AppConfig,
    /// Cache of loaded databases (name -> database)
    databases: Arc<RwLock<HashMap<String, Box<dyn Database + Send + Sync>>>>,
    /// Query result cache (query_string -> result)
    query_cache: Arc<RwLock<HashMap<String, CachedResult>>>,
}

/// Cached query result
#[derive(Clone)]
enum CachedResult {
    GeoLocation(Option<GeoLocation>),
    CdnProvider(Option<CdnProvider>),
}

impl DatabaseManager {
    /// Create a new database manager with configuration
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            databases: Arc::new(RwLock::new(HashMap::new())),
            query_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or load a database by name
    async fn get_or_load_database(&self, name: &str, db_type: DatabaseType) -> Result<()> {
        // Check if already loaded
        {
            let dbs = self.databases.read()
                .map_err(|e| NaliError::Other(format!("Failed to acquire read lock: {}", e)))?;
            if dbs.contains_key(name) {
                return Ok(());
            }
        }

        // Load the database
        log::info!("Loading database: {}", name);

        let mut db = DatabaseFactory::create(db_type);

        // Get database file path from config
        let db_path = self.config.get_database_path(name)?;

        // If database file doesn't exist, try to download it automatically
        if !db_path.exists() {
            log::warn!(
                "Database file not found: {:?}, attempting to download...",
                db_path
            );

            // Only auto-download for known databases (not custom ones)
            if let Some(db_info) = self
                .config
                .database
                .databases
                .iter()
                .find(|db| db.name == name || db.name_alias.contains(&name.to_string()))
            {
                if !db_info.download_urls.is_empty() {
                    eprintln!("Database file not found, automatically downloading {} database...", name);

                    let downloader = Downloader::new()?;
                    downloader.download_database(&self.config, name).await?;

                    eprintln!("✓ Database download complete\n");
                } else {
                    return Err(NaliError::DatabaseNotFound(format!(
                        "Database file not found and cannot be auto-downloaded: {:?}\nHint: Please run 'nali-rs --update {}' to manually download",
                        db_path, name
                    )));
                }
            } else {
                return Err(NaliError::DatabaseNotFound(format!(
                    "Database file not found: {:?}",
                    db_path
                )));
            }
        }

        // Load the database file
        db.load_from_file(db_path.to_str().unwrap())?;

        // Store in cache
        let mut dbs = self.databases.write()
            .map_err(|e| NaliError::Other(format!("Failed to acquire write lock: {}", e)))?;
        dbs.insert(name.to_string(), db);

        log::info!("Successfully loaded database: {}", name);
        Ok(())
    }

    /// Query IP geolocation
    ///
    /// Looks up geolocation information for the given IP address. The appropriate
    /// database (IPv4 or IPv6) is automatically selected based on the IP type.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address to query
    ///
    /// # Returns
    ///
    /// * `Ok(Some(GeoLocation))` - Geolocation information found
    /// * `Ok(None)` - IP not found in database
    /// * `Err(NaliError)` - Error occurred during query
    ///
    /// # Caching
    ///
    /// Query results are cached for improved performance on repeated queries.
    pub async fn query_ip(&self, ip: IpAddr) -> Result<Option<GeoLocation>> {
        // Check cache first
        let cache_key = format!("ip:{}", ip);
        {
            let cache = self.query_cache.read()
                .map_err(|e| NaliError::Other(format!("Failed to acquire cache read lock: {}", e)))?;
            if let Some(CachedResult::GeoLocation(result)) = cache.get(&cache_key) {
                return Ok(result.clone());
            }
        }

        // Determine which database to use
        let db_name = match ip {
            IpAddr::V4(_) => &self.config.database.ipv4_database,
            IpAddr::V6(_) => &self.config.database.ipv6_database,
        };

        let db_type = self.get_database_type(db_name)?;

        // Load database if needed
        self.get_or_load_database(db_name, db_type).await?;

        // Query
        let result = {
            let dbs = self.databases.read()
                .map_err(|e| NaliError::Other(format!("Failed to acquire database read lock: {}", e)))?;
            if let Some(db) = dbs.get(db_name) {
                db.lookup_ip(ip)?
            } else {
                None
            }
        };

        // Cache result
        {
            let mut cache = self.query_cache.write()
                .map_err(|e| NaliError::Other(format!("Failed to acquire cache write lock: {}", e)))?;
            cache.insert(cache_key, CachedResult::GeoLocation(result.clone()));
        }

        Ok(result)
    }

    /// Query CDN provider
    pub async fn query_cdn(&self, domain: &str) -> Result<Option<CdnProvider>> {
        // Check cache first
        let cache_key = format!("cdn:{}", domain);
        {
            let cache = self.query_cache.read()
                .map_err(|e| NaliError::Other(format!("Failed to acquire cache read lock: {}", e)))?;
            if let Some(CachedResult::CdnProvider(result)) = cache.get(&cache_key) {
                return Ok(result.clone());
            }
        }

        let db_name = &self.config.database.cdn_database;
        let db_type = DatabaseType::CDN;

        // Load database if needed
        self.get_or_load_database(db_name, db_type).await?;

        // Query
        let result = {
            let dbs = self.databases.read()
                .map_err(|e| NaliError::Other(format!("Failed to acquire database read lock: {}", e)))?;
            if let Some(db) = dbs.get(db_name) {
                db.lookup_cdn(domain)?
            } else {
                None
            }
        };

        // Cache result
        {
            let mut cache = self.query_cache.write()
                .map_err(|e| NaliError::Other(format!("Failed to acquire cache write lock: {}", e)))?;
            cache.insert(cache_key, CachedResult::CdnProvider(result.clone()));
        }

        Ok(result)
    }

    /// Get database type from name
    fn get_database_type(&self, name: &str) -> Result<DatabaseType> {
        match name {
            "qqwry" | "chunzhen" => Ok(DatabaseType::QQwry),
            "zxipv6wry" | "zxipv6" => Ok(DatabaseType::ZXIPv6Wry),
            "geoip" | "geoip2" | "geolite" => Ok(DatabaseType::GeoIP2),
            "ipip" => Ok(DatabaseType::IPIP),
            "ip2region" => Ok(DatabaseType::IP2Region),
            "dbip" => Ok(DatabaseType::DBIP),
            "ip2location" => Ok(DatabaseType::IP2Location),
            "cdn" => Ok(DatabaseType::CDN),
            _ => Err(NaliError::DatabaseNotFound(format!(
                "Unknown database type: {}",
                name
            ))),
        }
    }

    /// Clear query cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.query_cache.write() {
            cache.clear();
            log::info!("Query cache cleared");
        }
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        let db_count = self.databases.read().map(|dbs| dbs.len()).unwrap_or(0);
        let cache_count = self.query_cache.read().map(|cache| cache.len()).unwrap_or(0);
        (db_count, cache_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_manager_creation() {
        let config = AppConfig::default();
        let manager = DatabaseManager::new(config);
        let (db_count, cache_count) = manager.cache_stats();
        assert_eq!(db_count, 0);
        assert_eq!(cache_count, 0);
    }

    #[test]
    fn test_get_database_type() {
        let config = AppConfig::default();
        let manager = DatabaseManager::new(config);

        assert!(matches!(
            manager.get_database_type("qqwry"),
            Ok(DatabaseType::QQwry)
        ));
        assert!(matches!(
            manager.get_database_type("cdn"),
            Ok(DatabaseType::CDN)
        ));
        assert!(manager.get_database_type("unknown").is_err());
    }
}
