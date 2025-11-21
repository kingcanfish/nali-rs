//! ZX IPv6 database implementation core

use crate::database::{CdnProvider, Database, DatabaseType, GeoLocation};
use crate::error::Result;
use memmap2::Mmap;
use std::fs::File;
use std::net::IpAddr;

use super::reader::Reader;
use super::utils::{bytes3_to_u32, check_file};

/// ZX IPv6 database implementation
pub struct ZXIPv6Database {
    name: String,
    loaded: bool,
    mmap: Option<Mmap>,
    idx_start: u64,
    idx_end: u64,
    off_len: u8,
    ip_len: u8,
}

impl ZXIPv6Database {
    pub fn new() -> Self {
        Self {
            name: "zxipv6wry".to_string(),
            loaded: false,
            mmap: None,
            idx_start: 0,
            idx_end: 0,
            off_len: 0,
            ip_len: 0,
        }
    }

    /// Search index for IPv6 address (using first 64 bits)
    fn search_index(&self, ip: u64) -> Result<u32> {
        if let Some(ref mmap) = self.mmap {
            let ip_len = self.ip_len as u64;
            let entry_len = (self.off_len + self.ip_len) as u64;

            let mut l = self.idx_start;
            let mut r = self.idx_end;

            loop {
                let mid = (r - l) / entry_len / 2 * entry_len + l;
                let mid_ip = u64::from_le_bytes(
                    mmap[mid as usize..mid as usize + 8].try_into()?
                );

                // Check if we've narrowed down to one entry
                if r - l == entry_len {
                    // Check the right boundary
                    let r_ip = u64::from_le_bytes(
                        mmap[r as usize..r as usize + 8].try_into()?
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

    /// Lookup IPv6 address
    fn lookup_ipv6(&self, ip: u64) -> Result<Option<GeoLocation>> {
        if let Some(ref mmap) = self.mmap {
            // Search for the record offset
            let offset = self.search_index(ip)?;

            // Parse the record at offset using the same logic as QQwry
            let mut reader = Reader::new(mmap);
            let (country_bytes, area_bytes) = reader.parse(offset);

            // ZX IPv6 database uses UTF-8 encoding (not GBK like QQwry)
            // Convert bytes directly to UTF-8 strings
            log::debug!("Offset: 0x{:08x}", offset);
            log::debug!("Country bytes: {:?}", country_bytes);
            log::debug!("Country hex: {:02x?}", country_bytes);
            log::debug!("Area bytes: {:?}", area_bytes);
            log::debug!("Area hex: {:02x?}", area_bytes);

            let country = String::from_utf8_lossy(&country_bytes).to_string();
            let area = String::from_utf8_lossy(&area_bytes).to_string();

            log::debug!("Country string: '{}'", country);
            log::debug!("Area string: '{}'", area);

            // Clean up the strings
            let country = country.replace("CZ88.NET", "").trim().to_string();
            let area = area.replace("CZ88.NET", "").trim().to_string();

            // Reconstruct the full IPv6 address for display
            let ip_bytes = ip.to_be_bytes();
            let mut full_ipv6_bytes = [0u8; 16];
            full_ipv6_bytes[0..8].copy_from_slice(&ip_bytes);
            let ip_addr = IpAddr::V6(std::net::Ipv6Addr::from(full_ipv6_bytes));

            Ok(Some(GeoLocation {
                ip: ip_addr,
                country: if !country.is_empty() { Some(country) } else { None },
                region: None,
                city: None,
                isp: if !area.is_empty() { Some(area) } else { None },
                country_code: None,
                timezone: None,
                latitude: None,
                longitude: None,
            }))
        } else {
            Ok(None)
        }
    }
}

impl Database for ZXIPv6Database {
    fn name(&self) -> &str {
        &self.name
    }

    fn database_type(&self) -> DatabaseType {
        DatabaseType::ZXIPv6Wry
    }

    fn supports_ipv4(&self) -> bool {
        false
    }

    fn supports_ipv6(&self) -> bool {
        true
    }

    fn supports_cdn(&self) -> bool {
        false
    }

    fn lookup_ip(&self, ip: IpAddr) -> Result<Option<GeoLocation>> {
        match ip {
            IpAddr::V4(_) => {
                // ZX IPv6 database doesn't support IPv4
                Ok(None)
            }
            IpAddr::V6(ipv6) => {
                // ZX IPv6 only uses first 64 bits
                let ip_bytes = ipv6.octets();
                let ip_u64 = u64::from_be_bytes([
                    ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3],
                    ip_bytes[4], ip_bytes[5], ip_bytes[6], ip_bytes[7],
                ]);
                self.lookup_ipv6(ip_u64)
            }
        }
    }

    fn lookup_cdn(&self, _domain: &str) -> Result<Option<CdnProvider>> {
        // ZX IPv6 database doesn't support CDN lookup
        Ok(None)
    }

    fn is_loaded(&self) -> bool {
        self.loaded
    }

    fn load_from_file(&mut self, file_path: &str) -> Result<()> {
        log::info!("Loading ZX IPv6 database from: {}", file_path);

        // Open and memory map the file
        let file = File::open(file_path)
            .map_err(crate::error::NaliError::IoError)?;

        let mmap = unsafe { Mmap::map(&file) }
            .map_err(|e| crate::error::NaliError::parse(format!("Failed to memory map ZX IPv6 database: {}", e)))?;

        // Check file validity
        if !check_file(&mmap) {
            return Err(crate::error::NaliError::parse("Invalid ZX IPv6 database: file validation failed"));
        }

        // Read header
        let header = &mmap[0..24];
        let off_len = header[6];
        let ip_len = header[7];
        let counts = u64::from_le_bytes(mmap[8..16].try_into()?);
        let idx_start = u64::from_le_bytes(mmap[16..24].try_into()?);
        let idx_end = idx_start + counts * 11;

        self.off_len = off_len;
        self.ip_len = ip_len;
        self.idx_start = idx_start;
        self.idx_end = idx_end;
        self.mmap = Some(mmap);
        self.loaded = true;

        log::info!("Successfully loaded ZX IPv6 database: {} records", counts);

        Ok(())
    }
}

impl Default for ZXIPv6Database {
    fn default() -> Self {
        Self::new()
    }
}
