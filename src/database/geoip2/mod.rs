//! GeoIP2 database implementation
//!
//! This module implements support for MaxMind GeoIP2 database format,
//! which is an industry-standard IP geolocation database with multi-language support.

use crate::database::{CdnProvider, Database, DatabaseType, GeoLocation};
use crate::error::Result;
use maxminddb::geoip2;
use std::net::IpAddr;

/// GeoIP2 database implementation
pub struct GeoIP2Database {
    name: String,
    loaded: bool,
    reader: Option<maxminddb::Reader<Vec<u8>>>,
}

impl GeoIP2Database {
    pub fn new() -> Self {
        Self {
            name: "geoip2".to_string(),
            loaded: false,
            reader: None,
        }
    }

    /// Lookup IP address using GeoIP2
    fn lookup_internal(&self, ip: IpAddr) -> Result<Option<GeoLocation>> {
        if let Some(ref reader) = self.reader {
            // Query the database
            match reader.lookup::<geoip2::City>(ip) {
                Ok(city) => {
                    let country = city.country
                        .as_ref()
                        .and_then(|c| c.names.as_ref())
                        .and_then(|n| n.get("zh-CN").or_else(|| n.get("en")))
                        .map(|s| s.to_string());

                    let country_code = city.country
                        .as_ref()
                        .and_then(|c| c.iso_code)
                        .map(|s| s.to_string());

                    let city_name = city.city
                        .as_ref()
                        .and_then(|c| c.names.as_ref())
                        .and_then(|n| n.get("zh-CN").or_else(|| n.get("en")))
                        .map(|s| s.to_string());

                    let region = city.subdivisions
                        .as_ref()
                        .and_then(|subs| subs.last())
                        .and_then(|sub| sub.names.as_ref())
                        .and_then(|n| n.get("zh-CN").or_else(|| n.get("en")))
                        .map(|s| s.to_string());

                    let timezone = city.location
                        .as_ref()
                        .and_then(|l| l.time_zone)
                        .map(|s| s.to_string());

                    let latitude = city.location.as_ref().and_then(|l| l.latitude);
                    let longitude = city.location.as_ref().and_then(|l| l.longitude);

                    Ok(Some(GeoLocation {
                        ip,
                        country,
                        region,
                        city: city_name,
                        isp: None, // GeoIP2 City doesn't include ISP
                        country_code,
                        timezone,
                        latitude,
                        longitude,
                    }))
                }
                Err(maxminddb::MaxMindDBError::AddressNotFoundError(_)) => {
                    Ok(None)
                }
                Err(e) => {
                    Err(crate::error::NaliError::parse(format!("GeoIP2 lookup error: {}", e)))
                }
            }
        } else {
            Ok(None)
        }
    }
}

impl Database for GeoIP2Database {
    fn name(&self) -> &str {
        &self.name
    }

    fn database_type(&self) -> DatabaseType {
        DatabaseType::GeoIP2
    }

    fn supports_ipv4(&self) -> bool {
        true
    }

    fn supports_ipv6(&self) -> bool {
        true
    }

    fn supports_cdn(&self) -> bool {
        false
    }

    fn lookup_ip(&self, ip: IpAddr) -> Result<Option<GeoLocation>> {
        self.lookup_internal(ip)
    }

    fn lookup_cdn(&self, _domain: &str) -> Result<Option<CdnProvider>> {
        Ok(None)
    }

    fn is_loaded(&self) -> bool {
        self.loaded
    }

    fn load_from_file(&mut self, file_path: &str) -> Result<()> {
        log::info!("Loading GeoIP2 database from: {}", file_path);

        let reader = maxminddb::Reader::open_readfile(file_path)
            .map_err(|e| crate::error::NaliError::parse(format!("Failed to open GeoIP2 database: {}", e)))?;

        self.reader = Some(reader);
        self.loaded = true;

        log::info!("Successfully loaded GeoIP2 database from: {}", file_path);

        Ok(())
    }
}

impl Default for GeoIP2Database {
    fn default() -> Self {
        Self::new()
    }
}
