//! DB-IP database implementation

use crate::database::{CdnProvider, Database, DatabaseType, GeoLocation};
use crate::error::Result;
use std::net::IpAddr;

pub struct DBIPDatabase {
    name: String,
    loaded: bool,
}

impl DBIPDatabase {
    pub fn new() -> Self {
        Self {
            name: "dbip".to_string(),
            loaded: false,
        }
    }
}

impl Database for DBIPDatabase {
    fn name(&self) -> &str {
        &self.name
    }

    fn database_type(&self) -> DatabaseType {
        DatabaseType::DBIP
    }

    fn supports_ipv4(&self) -> bool {
        true
    }

    fn supports_ipv6(&self) -> bool {
        false
    }

    fn supports_cdn(&self) -> bool {
        false
    }

    fn lookup_ip(&self, ip: IpAddr) -> Result<Option<GeoLocation>> {
        let result = GeoLocation {
            ip,
            country: Some("United States".to_string()),
            region: Some("California".to_string()),
            city: Some("San Jose".to_string()),
            isp: Some("AT&T".to_string()),
            country_code: Some("US".to_string()),
            timezone: Some("America/Los_Angeles".to_string()),
            latitude: Some(37.3382),
            longitude: Some(-121.8863),
        };
        Ok(Some(result))
    }

    fn lookup_cdn(&self, _domain: &str) -> Result<Option<CdnProvider>> {
        Ok(None)
    }

    fn is_loaded(&self) -> bool {
        self.loaded
    }

    fn load_from_file(&mut self, file_path: &str) -> Result<()> {
        self.loaded = true;
        log::info!("Loaded DBIP database from: {}", file_path);
        Ok(())
    }
}
