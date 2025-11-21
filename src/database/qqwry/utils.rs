//! Utility functions for QQwry database

/// Convert 3 bytes to u32 (little-endian)
pub fn bytes3_to_u32(data: &[u8]) -> u32 {
    let i = (data[0] as u32) & 0xff;
    let i = i | ((data[1] as u32) << 8) & 0xff00;
    let i = i | ((data[2] as u32) << 16) & 0xff0000;
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes3_to_u32() {
        let data = [0x01, 0x02, 0x03];
        let result = bytes3_to_u32(&data);
        assert_eq!(result, 0x00030201);
    }
}
