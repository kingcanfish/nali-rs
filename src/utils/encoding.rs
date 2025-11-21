//! Character encoding utilities
//!
//! Provides functions for converting between different character encodings,
//! especially GBK to UTF-8 which is needed for Chinese IP databases.

use crate::error::Result;
use encoding_rs::GBK;

/// Convert GBK encoded bytes to UTF-8 string
pub fn gbk_to_utf8(data: &[u8]) -> Result<String> {
    // Decode GBK to UTF-8, using replacement character for invalid sequences
    // This matches the Go version's approach: simplifiedchinese.GBK.NewDecoder().String()
    let (cow, _encoding_used, had_errors) = GBK.decode(data);

    if had_errors {
        log::debug!("GBK decoding had errors for bytes: {:?}", data);
    }

    // Simply convert to string and trim, just like Go version does
    let result = cow.trim().to_string();

    log::debug!("GBK decoded '{}' from bytes: {:?}", result, data);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gbk_to_utf8() {
        // GBK encoding of "中国" (China)
        let gbk_bytes = vec![0xD6, 0xD0, 0xB9, 0xFA];
        let result = gbk_to_utf8(&gbk_bytes).unwrap();
        assert_eq!(result, "中国");
    }
}
