//! IPIP translation tables for ID to string conversion

use super::header::IPIPHeader;

/// IPIP database translation tables
#[derive(Debug)]
pub struct IPIPTranslationTables {
    countries: Vec<String>,
    regions: Vec<String>,
    cities: Vec<String>,
    isps: Vec<String>,
}

impl IPIPTranslationTables {
    /// Parse translation tables from data
    pub fn parse(data: &[u8], header: &IPIPHeader) -> crate::error::Result<Self> {
        let mut countries = Vec::new();
        let mut regions = Vec::new();
        let mut cities = Vec::new();
        let mut isps = Vec::new();

        // IPIP databases typically have a text section after the index
        let text_start = header.index_end + (header.index_count() * 16) as u32;

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

        Ok(Self {
            countries,
            regions,
            cities,
            isps,
        })
    }

    /// Translate ID to string
    pub fn translate(&self, id: u16, table_name: &str) -> String {
        match table_name {
            "countries" => {
                if (id as usize) < self.countries.len() {
                    self.countries[id as usize].clone()
                } else {
                    "Unknown".to_string()
                }
            }
            "regions" => {
                if (id as usize) < self.regions.len() {
                    self.regions[id as usize].clone()
                } else {
                    "Unknown".to_string()
                }
            }
            "cities" => {
                if (id as usize) < self.cities.len() {
                    self.cities[id as usize].clone()
                } else {
                    "Unknown".to_string()
                }
            }
            "isps" => {
                if (id as usize) < self.isps.len() {
                    self.isps[id as usize].clone()
                } else {
                    "Unknown".to_string()
                }
            }
            _ => "Unknown".to_string()
        }
    }
}
