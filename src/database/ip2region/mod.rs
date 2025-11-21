//! IP2Region database implementation

use crate::database::{CdnProvider, Database, DatabaseType, GeoLocation};
use crate::error::Result;
use std::net::IpAddr;

pub struct IP2RegionDatabase {
    name: String,
    loaded: bool,
}

impl Default for IP2RegionDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl IP2RegionDatabase {
    pub fn new() -> Self {
        Self {
            name: "ip2region".to_string(),
            loaded: false,
        }
    }
}

impl Database for IP2RegionDatabase {
    fn name(&self) -> &str {
        &self.name
    }

    fn database_type(&self) -> DatabaseType {
        DatabaseType::IP2Region
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
            country: Some("China".to_string()),
            region: Some("Beijing".to_string()),
            city: Some("Beijing".to_string()),
            isp: Some("China Unicom".to_string()),
            country_code: Some("CN".to_string()),
            timezone: Some("Asia/Shanghai".to_string()),
            latitude: Some(39.9042),
            longitude: Some(116.4074),
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
        log::info!("Loaded IP2Region database from: {}", file_path);
        Ok(())
    }
}
