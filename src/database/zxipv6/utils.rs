//! Utility functions for ZX IPv6 database

/// Convert 3 bytes to u32 (little-endian)
pub fn bytes3_to_u32(data: &[u8]) -> u32 {
    let i = (data[0] as u32) & 0xff;
    let i = i | ((data[1] as u32) << 8) & 0xff00;
    
    i | ((data[2] as u32) << 16) & 0xff0000
}

/// Check if file is valid ZX IPv6 database
pub fn check_file(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }
    if &data[0..4] != b"IPDB" {
        return false;
    }

    if data.len() < 24 {
        return false;
    }

    let start = u64::from_le_bytes(data[16..24].try_into().unwrap_or([0u8; 8]));
    let counts = u64::from_le_bytes(data[8..16].try_into().unwrap_or([0u8; 8]));
    let end = start + counts * 11;

    if start >= end || (data.len() as u64) < end {
        return false;
    }

    true
}
