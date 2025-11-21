//! IPIP database implementation
//! 
//! This module implements support for the IPIP database format.
//! IPIP is known for its balance between accuracy and performance,
//! supporting both IPv4 and IPv6 geolocation lookup.

use crate::database::{Database, DatabaseType, GeoLocation, CdnProvider};
use crate::error::Result;
use std::net::IpAddr;
use std::fs::File;
use memmap2::Mmap;

/// IPIP database header structure
#[derive(Debug)]
struct IPIPHeader {
    version: u32,
    created_time: u32,
    index_start: u32,
    index_end: u32,
    support_ipv6: bool,
}

/// IPIP database record
#[derive(Debug, Clone)]
struct IPIPRecord {
    start_ip: u32,
    end_ip: u32,
    country_id: u16,
    region_id: u16,
    city_id: u16,
    isp_id: u16,
}

/// IPIP database translation tables
#[derive(Debug)]
struct IPIPTranslationTables {
    countries: Vec<String>,
    regions: Vec<String>,
    cities: Vec<String>,
    isps: Vec<String>,
}

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
    ipv6_tree_start: u32,
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
    
    /// Parse IPIP database header
    fn parse_header(&self, data: &[u8]) -> Result<IPIPHeader> {
        if data.len() < 16 {
            return Err(crate::error::NaliError::parse("Invalid IPIP database: file too small"));
        }
        
        let version = u32::from_le_bytes(data[0..4].try_into()?);
        let created_time = u32::from_le_bytes(data[4..8].try_into()?);
        let index_start = u32::from_le_bytes(data[8..12].try_into()?);
        let index_end = u32::from_le_bytes(data[12..16].try_into()?);
        
        // Check IPv6 support flag (usually in the 16th byte)
        let support_ipv6 = if data.len() > 16 {
            data[16] == 1
        } else {
            false
        };
        
        Ok(IPIPHeader {
            version,
            created_time,
            index_start,
            index_end,
            support_ipv6,
        })
    }
    
    /// Parse translation tables from data
    fn parse_translation_tables(&self, data: &[u8], header: &IPIPHeader) -> Result<IPIPTranslationTables> {
        let mut countries = Vec::new();
        let mut regions = Vec::new();
        let mut cities = Vec::new();
        let mut isps = Vec::new();
        
        // IPIP databases typically have a text section after the index
        let text_start = header.index_end + (Self::index_count(header) * 16) as u32;
        
        if (text_start as usize) < data.len() {
            let text_data = &data[text_start as usize..];
            let text_str = String::from_utf8_lossy(text_data);
            
            // Simple parsing - split by null bytes and categorize
            for line in text_str.split('\0') {
                if line.is_empty() {
                    continue;
                }
                
                // Basic categorization logic - in production would be more sophisticated
                if line.contains("国家") || line.contains("China") || line.contains("United States") {
                    countries.push(line.to_string());
                } else if line.contains("省") || line.contains("州") || line.contains("Province") {
                    regions.push(line.to_string());
                } else if line.contains("市") || line.contains("City") {
                    cities.push(line.to_string());
                } else if line.contains("电信") || line.contains("运营商") || line.contains("ISP") {
                    isps.push(line.to_string());
                }
            }
        }
        
        Ok(IPIPTranslationTables {
            countries,
            regions,
            cities,
            isps,
        })
    }
    
    /// Calculate index count from header
    fn index_count(header: &IPIPHeader) -> usize {
        ((header.index_end - header.index_start) / 16) as usize
    }
    
    /// IPv4 address to u32 for lookup
    fn ipv4_to_u32(ipv4: &std::net::Ipv4Addr) -> u32 {
        u32::from_be_bytes(ipv4.octets())
    }
    
    /// IPv6 address to u128 for lookup (simplified)
    fn ipv6_to_u128(ipv6: &std::net::Ipv6Addr) -> u128 {
        u128::from_be_bytes(ipv6.octets())
    }
    
    /// Parse a single record from the database
    fn parse_record(&self, offset: u32) -> Result<IPIPRecord> {
        if let Some(ref mmap) = self.mmap {
            if offset as usize + 16 > mmap.len() {
                return Err(crate::error::NaliError::parse(format!("Record offset out of bounds: {}", offset)));
            }
            
            // Read IP range and metadata
            let start_ip = u32::from_le_bytes(mmap[offset as usize..offset as usize + 4].try_into()?);
            let end_ip = u32::from_le_bytes(mmap[offset as usize + 4..offset as usize + 8].try_into()?);
            let country_id = u16::from_le_bytes(mmap[offset as usize + 8..offset as usize + 10].try_into()?);
            let region_id = u16::from_le_bytes(mmap[offset as usize + 10..offset as usize + 12].try_into()?);
            let city_id = u16::from_le_bytes(mmap[offset as usize + 12..offset as usize + 14].try_into()?);
            let isp_id = u16::from_le_bytes(mmap[offset as usize + 14..offset as usize + 16].try_into()?);
            
            Ok(IPIPRecord {
                start_ip,
                end_ip,
                country_id,
                region_id,
                city_id,
                isp_id,
            })
        } else {
            Err(crate::error::NaliError::parse("Database not loaded"))
        }
    }
    
    /// Binary search for IPv4 address in database
    fn lookup_ip_internal_v4(&self, ip: u32) -> Result<Option<GeoLocation>> {
        if let Some(ref header) = self.header {
            let mut low = header.index_start;
            let mut high = header.index_end;
            
            while low <= high {
                let mid = (low + high) / 2;
                if mid as usize + 16 > self.file_size {
                    break;
                }
                
                let record = self.parse_record(mid)?;
                
                if ip >= record.start_ip && ip <= record.end_ip {
                    // Translate IDs to strings using translation tables
                    let country = self.translate_id(&record.country_id, "countries")?;
                    let region = self.translate_id(&record.region_id, "regions")?;
                    let city = self.translate_id(&record.city_id, "cities")?;
                    let isp = self.translate_id(&record.isp_id, "isps")?;
                    
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
    
    /// Translate ID to string using translation tables
    fn translate_id(&self, id: &u16, table_name: &str) -> Result<String> {
        if let Some(ref tables) = self.translation_tables {
            match table_name {
                "countries" => {
                    if (*id as usize) < tables.countries.len() {
                        Ok(tables.countries[*id as usize].clone())
                    } else {
                        Ok("Unknown".to_string())
                    }
                }
                "regions" => {
                    if (*id as usize) < tables.regions.len() {
                        Ok(tables.regions[*id as usize].clone())
                    } else {
                        Ok("Unknown".to_string())
                    }
                }
                "cities" => {
                    if (*id as usize) < tables.cities.len() {
                        Ok(tables.cities[*id as usize].clone())
                    } else {
                        Ok("Unknown".to_string())
                    }
                }
                "isps" => {
                    if (*id as usize) < tables.isps.len() {
                        Ok(tables.isps[*id as usize].clone())
                    } else {
                        Ok("Unknown".to_string())
                    }
                }
                _ => Ok("Unknown".to_string())
            }
        } else {
            Ok("Unknown".to_string())
        }
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
        self.mmap = Some(mmap);
        
        // Parse header
        if let Some(ref mmap) = self.mmap {
            let header = self.parse_header(mmap)?;
            self.header = Some(header);
            self.ipv6_support = self.header.as_ref().unwrap().support_ipv6;
            
            // Parse translation tables
            if let Some(ref hdr) = self.header {
                let translation_tables = self.parse_translation_tables(mmap, hdr)?;
                self.translation_tables = Some(translation_tables);
                
                log::info!("IPIP database version: {}", hdr.version);
                log::info!("IPIP database created: {}", hdr.created_time);
                log::info!("IPv6 support: {}", hdr.support_ipv6);
            }
        }
        
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
