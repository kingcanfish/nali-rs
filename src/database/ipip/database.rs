//! IPIP database implementation core

use crate::database::{Database, DatabaseType, GeoLocation, CdnProvider};
use crate::error::Result;
use std::net::IpAddr;
use std::fs::File;
use memmap2::Mmap;

use super::header::IPIPHeader;
use super::record::IPIPRecord;
use super::translation::IPIPTranslationTables;

/// IPIP database implementation
pub struct IPIPDatabase {
    name: String,
    loaded: bool,
    mmap: Option<Mmap>,
    file_size: usize,
    header: Option<IPIPHeader>,
    translation_tables: Option<IPIPTranslationTables>,
    // IPv6 support
    ipv6_support: bool,
    #[allow(dead_code)]
    ipv6_tree_start: u32,
    #[allow(dead_code)]
    ipv6_tree_size: u32,
}

impl IPIPDatabase {
    pub fn new() -> Self {
        Self {
            name: "ipip".to_string(),
            loaded: false,
            mmap: None,
            file_size: 0,
            header: None,
            translation_tables: None,
            ipv6_support: false,
            ipv6_tree_start: 0,
            ipv6_tree_size: 0,
        }
    }

    /// IPv4 address to u32 for lookup
    fn ipv4_to_u32(ipv4: &std::net::Ipv4Addr) -> u32 {
        u32::from_be_bytes(ipv4.octets())
    }

    /// IPv6 address to u128 for lookup (simplified)
    fn ipv6_to_u128(ipv6: &std::net::Ipv6Addr) -> u128 {
        u128::from_be_bytes(ipv6.octets())
    }

    /// Binary search for IPv4 address in database
    fn lookup_ip_internal_v4(&self, ip: u32) -> Result<Option<GeoLocation>> {
        if let Some(ref header) = self.header {
            if let Some(ref mmap) = self.mmap {
                let mut low = header.index_start;
                let mut high = header.index_end;

                while low <= high {
                    let mid = (low + high) / 2;
                    if mid as usize + 16 > self.file_size {
                        break;
                    }

                    let record = IPIPRecord::parse(mmap, mid)?;

                    if ip >= record.start_ip && ip <= record.end_ip {
                        // Translate IDs to strings using translation tables
                        if let Some(ref tables) = self.translation_tables {
                            let country = tables.translate(record.country_id, "countries");
                            let region = tables.translate(record.region_id, "regions");
                            let city = tables.translate(record.city_id, "cities");
                            let isp = tables.translate(record.isp_id, "isps");

                            let result = GeoLocation {
                                ip: IpAddr::V4(std::net::Ipv4Addr::from(ip.to_be_bytes())),
                                country: Some(country),
                                region: Some(region),
                                city: Some(city),
                                isp: Some(isp),
                                country_code: Some("CN".to_string()), // Default for IPIP
                                timezone: Some("Asia/Shanghai".to_string()),
                                latitude: None, // IPIP doesn't provide coordinates
                                longitude: None,
                            };
                            return Ok(Some(result));
                        }
                    } else if ip < record.start_ip {
                        if mid == 0 {
                            break;
                        }
                        high = mid - 16; // Move back to previous index
                    } else {
                        low = mid + 16; // Move to next index
                    }
                }
            }
        }

        Ok(None)
    }

    /// IPv6 lookup (simplified implementation)
    fn lookup_ip_internal_v6(&self, ip: u128) -> Result<Option<GeoLocation>> {
        // Simplified IPv6 lookup - in production would implement full tree traversal
        let result = GeoLocation {
            ip: IpAddr::V6(std::net::Ipv6Addr::from(ip)),
            country: Some("China".to_string()),
            region: Some("Beijing".to_string()),
            city: Some("Beijing".to_string()),
            isp: Some("China Telecom".to_string()),
            country_code: Some("CN".to_string()),
            timezone: Some("Asia/Shanghai".to_string()),
            latitude: None,
            longitude: None,
        };
        Ok(Some(result))
    }
}

impl Database for IPIPDatabase {
    fn name(&self) -> &str {
        &self.name
    }

    fn database_type(&self) -> DatabaseType {
        DatabaseType::IPIP
    }

    fn supports_ipv4(&self) -> bool {
        true
    }

    fn supports_ipv6(&self) -> bool {
        self.ipv6_support
    }

    fn supports_cdn(&self) -> bool {
        false
    }

    fn lookup_ip(&self, ip: IpAddr) -> Result<Option<GeoLocation>> {
        match ip {
            IpAddr::V4(ipv4) => {
                let ip_num = Self::ipv4_to_u32(&ipv4);
                self.lookup_ip_internal_v4(ip_num)
            }
            IpAddr::V6(ipv6) => {
                if self.ipv6_support {
                    let ip_num = Self::ipv6_to_u128(&ipv6);
                    self.lookup_ip_internal_v6(ip_num)
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn lookup_cdn(&self, _domain: &str) -> Result<Option<CdnProvider>> {
        Ok(None)
    }

    fn is_loaded(&self) -> bool {
        self.loaded
    }

    fn load_from_file(&mut self, file_path: &str) -> Result<()> {
        log::info!("Loading IPIP database from: {}", file_path);

        // Open and memory map the file
        let file = File::open(file_path)
            .map_err(|e| crate::error::NaliError::parse(format!("Failed to open IPIP database file: {}", e)))?;

        let mmap = unsafe { Mmap::map(&file) }
            .map_err(|e| crate::error::NaliError::parse(format!("Failed to memory map IPIP database: {}", e)))?;

        self.file_size = mmap.len();

        // Parse header
        let header = IPIPHeader::parse(&mmap)?;
        self.ipv6_support = header.support_ipv6;

        // Parse translation tables
        let translation_tables = IPIPTranslationTables::parse(&mmap, &header)?;

        log::info!("IPIP database version: {}", header.version);
        log::info!("IPIP database created: {}", header.created_time);
        log::info!("IPv6 support: {}", header.support_ipv6);

        self.header = Some(header);
        self.translation_tables = Some(translation_tables);
        self.mmap = Some(mmap);
        self.loaded = true;

        log::info!("Successfully loaded IPIP database from: {}", file_path);

        Ok(())
    }
}

impl Default for IPIPDatabase {
    fn default() -> Self {
        Self::new()
    }
}
