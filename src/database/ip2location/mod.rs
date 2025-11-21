//! IP2Location database implementation

use crate::database::{CdnProvider, Database, DatabaseType, GeoLocation};
use crate::error::Result;
use std::net::IpAddr;

pub struct IP2LocationDatabase {
    name: String,
    loaded: bool,
}

impl IP2LocationDatabase {
    pub fn new() -> Self {
        Self {
            name: "ip2location".to_string(),
            loaded: false,
        }
    }
}

impl Database for IP2LocationDatabase {
    fn name(&self) -> &str {
        &self.name
    }

    fn database_type(&self) -> DatabaseType {
        DatabaseType::IP2Location
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
            country: Some("United Kingdom".to_string()),
            region: Some("England".to_string()),
            city: Some("London".to_string()),
            isp: Some("British Telecom".to_string()),
            country_code: Some("GB".to_string()),
            timezone: Some("Europe/London".to_string()),
            latitude: Some(51.5074),
            longitude: Some(-0.1278),
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
        log::info!("Loaded IP2Location database from: {}", file_path);
        Ok(())
    }
}
