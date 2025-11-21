//! Domain matching utilities for CDN database

use regex::Regex;
use super::entry::CdnEntry;

/// Extract base domain from a full domain
/// e.g., "www.example.com" -> "example.com"
pub fn extract_base_domain(domain: &str) -> Vec<String> {
    let mut candidates = Vec::new();

    // Add the full domain
    candidates.push(domain.to_lowercase());

    // Split by dots and try different combinations
    let parts: Vec<&str> = domain.split('.').collect();
    if parts.len() >= 2 {
        // example.com from www.example.com
        let base = parts[parts.len() - 2..].join(".");
        candidates.push(base.to_lowercase());
    }

    candidates
}

/// Convert wildcard pattern to regex
/// e.g., "*.example.com" -> "^.*\\.example\\.com$"
pub fn wildcard_to_regex(pattern: &str) -> String {
    let mut result = String::from("^");
    for ch in pattern.chars() {
        match ch {
            '*' => result.push_str(".*"),
            '?' => result.push('.'),
            '.' => result.push_str("\\."),
            '\\' => result.push_str("\\\\"),
            '+' => result.push_str("\\+"),
            '^' => result.push_str("\\^"),
            '$' => result.push_str("\\$"),
            '(' => result.push_str("\\("),
            ')' => result.push_str("\\)"),
            '[' => result.push_str("\\["),
            ']' => result.push_str("\\]"),
            '{' => result.push_str("\\{"),
            '}' => result.push_str("\\}"),
            '|' => result.push_str("\\|"),
            _ => result.push(ch),
        }
    }
    result.push('$');
    result
}

/// Check if a domain matches any regex patterns
pub fn match_regex<'a>(
    domain: &str,
    regex_matches: &'a [(Regex, CdnEntry)]
) -> Option<&'a CdnEntry> {
    for (regex, entry) in regex_matches {
        if regex.is_match(domain) {
            return Some(entry);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_base_domain() {
        let candidates = extract_base_domain("www.example.com");
        assert!(candidates.contains(&"www.example.com".to_string()));
        assert!(candidates.contains(&"example.com".to_string()));
    }

    #[test]
    fn test_wildcard_to_regex() {
        let regex = wildcard_to_regex("*.example.com");
        assert_eq!(regex, "^.*\\.example\\.com$");
    }
}
