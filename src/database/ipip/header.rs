//! IPIP database header structure

/// IPIP database header
#[derive(Debug)]
pub struct IPIPHeader {
    pub version: u32,
    pub created_time: u32,
    pub index_start: u32,
    pub index_end: u32,
    pub support_ipv6: bool,
}

impl IPIPHeader {
    /// Parse header from raw data
    pub fn parse(data: &[u8]) -> crate::error::Result<Self> {
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

        Ok(Self {
            version,
            created_time,
            index_start,
            index_end,
            support_ipv6,
        })
    }

    /// Calculate index count
    pub fn index_count(&self) -> usize {
        ((self.index_end - self.index_start) / 16) as usize
    }
}
