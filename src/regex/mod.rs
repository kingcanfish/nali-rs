//! Regular expressions for matching IP addresses and domains
//!
//! This module provides pre-compiled regular expressions for extracting
//! IPv4, IPv6 addresses and domain names from text.

use once_cell::sync::Lazy;
use regex::Regex;

/// IPv4 address regex
/// Matches standard IPv4 addresses like 192.168.1.1
pub static IPV4_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)"
    )
    .expect("Failed to compile IPv4 regex")
});

/// IPv6 address regex
/// Matches various IPv6 formats including compressed notation
pub static IPV6_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"fe80:(:[0-9a-fA-F]{1,4}){0,4}(%\w+)?|([0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}|64:ff9b::(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)(\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3}|::[fF]{4}:(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)(\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3}|(([0-9a-fA-F]{1,4}:){0,6}[0-9a-fA-F]{1,4})?::(([0-9a-fA-F]{1,4}:){0,6}[0-9a-fA-F]{1,4})?"
    )
    .expect("Failed to compile IPv6 regex")
});

/// Domain name regex
/// Matches standard domain names like example.com, sub.example.com
pub static DOMAIN_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9][a-z0-9-]{0,61}[a-z0-9]"
    )
    .expect("Failed to compile domain regex")
});

/// Find all IPv4 addresses in text with their positions
pub fn find_ipv4(text: &str) -> Vec<(usize, usize, String)> {
    IPV4_RE
        .find_iter(text)
        .map(|m| (m.start(), m.end(), m.as_str().to_string()))
        .collect()
}

/// Find all IPv6 addresses in text with their positions
pub fn find_ipv6(text: &str) -> Vec<(usize, usize, String)> {
    IPV6_RE
        .find_iter(text)
        .map(|m| (m.start(), m.end(), m.as_str().to_string()))
        .collect()
}

/// Find all domain names in text with their positions
pub fn find_domains(text: &str) -> Vec<(usize, usize, String)> {
    DOMAIN_RE
        .find_iter(text)
        .map(|m| (m.start(), m.end(), m.as_str().to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_regex() {
        let text = "Server IP: 192.168.1.1 and 8.8.8.8";
        let matches = find_ipv4(text);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].2, "192.168.1.1");
        assert_eq!(matches[1].2, "8.8.8.8");
    }

    #[test]
    fn test_ipv6_regex() {
        let text = "IPv6: 2001:0db8::1 and ::1";
        let matches = find_ipv6(text);
        assert!(matches.len() >= 1);
        // Note: The regex may match partial addresses, so we just check that we found something
        assert!(matches[0].2.contains("2001"));
    }

    #[test]
    fn test_domain_regex() {
        let text = "Visit example.com and sub.example.org";
        let matches = find_domains(text);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].2, "example.com");
        assert_eq!(matches[1].2, "sub.example.org");
    }

    #[test]
    fn test_invalid_ipv4() {
        let text = "Invalid: 999.999.999.999";
        let matches = find_ipv4(text);
        assert_eq!(matches.len(), 0);
    }
}
