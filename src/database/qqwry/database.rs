//! QQwry database implementation core

use crate::database::{CdnProvider, Database, DatabaseType, GeoLocation};
use crate::error::Result;
use crate::utils::encoding::gbk_to_utf8;
use memmap2::Mmap;
use std::fs::File;
use std::net::IpAddr;

use super::reader::Reader;
use super::utils::bytes3_to_u32;

/// QQwry database implementation
pub struct QQwryDatabase {
    name: String,
    loaded: bool,
    mmap: Option<Mmap>,
    idx_start: u32,
    idx_end: u32,
}

impl QQwryDatabase {
    pub fn new() -> Self {
        Self {
            name: "qqwry".to_string(),
            loaded: false,
            mmap: None,
            idx_start: 0,
            idx_end: 0,
        }
    }

    /// Search index for IPv4 address
    fn search_index(&self, ip: u32) -> Result<u32> {
        if let Some(ref mmap) = self.mmap {
            let ip_len = 4u32;
            let entry_len = 7u32; // 4 bytes IP + 3 bytes offset

            let mut l = self.idx_start;
            let mut r = self.idx_end;

            loop {
                let mid = (r - l) / entry_len / 2 * entry_len + l;
                let mid_ip = u32::from_le_bytes(
                    mmap[mid as usize..mid as usize + 4].try_into()?
                );

                // Check if we've narrowed down to one entry
                if r - l == entry_len {
                    // Check the right boundary
                    let r_ip = u32::from_le_bytes(
                        mmap[r as usize..r as usize + 4].try_into()?
                    );

                    let offset_pos = if ip >= r_ip { r } else { mid };
                    return Ok(bytes3_to_u32(
                        &mmap[offset_pos as usize + ip_len as usize..offset_pos as usize + entry_len as usize]
                    ));
                }

                if mid_ip > ip {
                    r = mid;
                } else if mid_ip < ip {
                    l = mid;
                } else {
                    // Exact match
                    return Ok(bytes3_to_u32(
                        &mmap[mid as usize + ip_len as usize..mid as usize + entry_len as usize]
                    ));
                }
            }
        } else {
            Err(crate::error::NaliError::parse("Database not loaded"))
        }
    }

    /// Lookup IPv4 address
    fn lookup_ipv4(&self, ip: u32) -> Result<Option<GeoLocation>> {
        if let Some(ref mmap) = self.mmap {
            // Search for the record offset
            let offset = self.search_index(ip)?;

            // Parse the record at offset
            let mut reader = Reader::new(mmap);
            // Skip the end IP (4 bytes) and parse location
            let (country_bytes, area_bytes) = reader.parse(offset + 4);

            // Convert GBK to UTF-8
            let country = gbk_to_utf8(&country_bytes)?;
            let area = gbk_to_utf8(&area_bytes)?;

            // Clean up the strings
            let country = country.replace("CZ88.NET", "").trim().to_string();
            let area = area.replace("CZ88.NET", "").trim().to_string();

            let ip_addr = IpAddr::V4(std::net::Ipv4Addr::from(ip));

            Ok(Some(GeoLocation {
                ip: ip_addr,
                country: if !country.is_empty() { Some(country) } else { None },
                region: None,
                city: None,
                isp: if !area.is_empty() { Some(area) } else { None },
                country_code: Some("CN".to_string()),
                timezone: Some("Asia/Shanghai".to_string()),
                latitude: None,
                longitude: None,
            }))
        } else {
            Ok(None)
        }
    }
}

impl Database for QQwryDatabase {
    fn name(&self) -> &str {
        &self.name
    }

    fn database_type(&self) -> DatabaseType {
        DatabaseType::QQwry
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
        match ip {
            IpAddr::V4(ipv4) => {
                let ip_num = u32::from_be_bytes(ipv4.octets());
                self.lookup_ipv4(ip_num)
            }
            IpAddr::V6(_) => {
                // QQwry doesn't support IPv6
                Ok(None)
            }
        }
    }

    fn lookup_cdn(&self, _domain: &str) -> Result<Option<CdnProvider>> {
        // QQwry database doesn't support CDN lookup
        Ok(None)
    }

    fn is_loaded(&self) -> bool {
        self.loaded
    }

    fn load_from_file(&mut self, file_path: &str) -> Result<()> {
        log::info!("Loading QQwry database from: {}", file_path);

        // Open and memory map the file
        let file = File::open(file_path)
            .map_err(|e| crate::error::NaliError::IoError(e))?;

        let mmap = unsafe { Mmap::map(&file) }
            .map_err(|e| crate::error::NaliError::parse(format!("Failed to memory map QQwry database: {}", e)))?;

        // Check file validity
        if mmap.len() < 8 {
            return Err(crate::error::NaliError::parse("Invalid QQwry database: file too small"));
        }

        // Read header
        let idx_start = u32::from_le_bytes(mmap[0..4].try_into()?);
        let idx_end = u32::from_le_bytes(mmap[4..8].try_into()?);

        // Validate header
        if idx_start >= idx_end || mmap.len() < (idx_end + 7) as usize {
            return Err(crate::error::NaliError::parse("Invalid QQwry database: header validation failed"));
        }

        self.idx_start = idx_start;
        self.idx_end = idx_end;
        self.mmap = Some(mmap);
        self.loaded = true;

        let record_count = (idx_end - idx_start) / 7 + 1;
        log::info!("Successfully loaded QQwry database: {} records", record_count);

        Ok(())
    }
}

impl Default for QQwryDatabase {
    fn default() -> Self {
        Self::new()
    }
}
