//! IPIP database record structure

/// IPIP database record
#[derive(Debug, Clone)]
pub struct IPIPRecord {
    pub start_ip: u32,
    pub end_ip: u32,
    pub country_id: u16,
    pub region_id: u16,
    pub city_id: u16,
    pub isp_id: u16,
}

impl IPIPRecord {
    /// Parse a single record from data at given offset
    pub fn parse(data: &[u8], offset: u32) -> crate::error::Result<Self> {
        if offset as usize + 16 > data.len() {
            return Err(crate::error::NaliError::parse(format!("Record offset out of bounds: {}", offset)));
        }

        let start_ip = u32::from_le_bytes(data[offset as usize..offset as usize + 4].try_into()?);
        let end_ip = u32::from_le_bytes(data[offset as usize + 4..offset as usize + 8].try_into()?);
        let country_id = u16::from_le_bytes(data[offset as usize + 8..offset as usize + 10].try_into()?);
        let region_id = u16::from_le_bytes(data[offset as usize + 10..offset as usize + 12].try_into()?);
        let city_id = u16::from_le_bytes(data[offset as usize + 12..offset as usize + 14].try_into()?);
        let isp_id = u16::from_le_bytes(data[offset as usize + 14..offset as usize + 16].try_into()?);

        Ok(Self {
            start_ip,
            end_ip,
            country_id,
            region_id,
            city_id,
            isp_id,
        })
    }
}
