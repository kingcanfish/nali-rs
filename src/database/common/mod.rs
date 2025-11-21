//! CDN database implementation
//!
//! This module implements support for CDN provider identification
//! based on domain names using YAML configuration.

use crate::database::{CdnProvider, Database, DatabaseType, GeoLocation};
use crate::error::{NaliError, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::IpAddr;

/// CDN database entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnEntry {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
}

/// CDN database structure
pub struct CDNDatabase {
    name: String,
    loaded: bool,
    /// Exact domain matches (domain -> CdnEntry)
    exact_matches: HashMap<String, CdnEntry>,
    /// Regex pattern matches (pattern -> CdnEntry)
    regex_matches: Vec<(Regex, CdnEntry)>,
}

impl CDNDatabase {
    pub fn new() -> Self {
        Self {
            name: "cdn".to_string(),
            loaded: false,
            exact_matches: HashMap::new(),
            regex_matches: Vec::new(),
        }
    }

    /// Extract base domain from a full domain
    /// e.g., "www.example.com" -> "example.com"
    fn extract_base_domain(domain: &str) -> Vec<String> {
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

    /// Check if a domain matches any regex patterns
    fn match_regex(&self, domain: &str) -> Option<&CdnEntry> {
        for (regex, entry) in &self.regex_matches {
            if regex.is_match(domain) {
                return Some(entry);
            }
        }
        None
    }

    /// Parse YAML CDN database file
    fn parse_yaml(&mut self, content: &str) -> Result<()> {
        // Parse YAML as HashMap
        let data: HashMap<String, CdnEntry> = serde_yaml::from_str(content)
            .map_err(|e| NaliError::YamlError(format!("解析CDN数据库失败: {}", e)))?;

        for (pattern, entry) in data {
            // Check if pattern is a wildcard or regex
            if pattern.contains('*') || pattern.contains('?') {
                // Convert wildcard to regex
                let regex_pattern = Self::wildcard_to_regex(&pattern);
                match Regex::new(&regex_pattern) {
                    Ok(regex) => {
                        self.regex_matches.push((regex, entry));
                        log::debug!("Added CDN wildcard pattern: {} -> {}", pattern, regex_pattern);
                    }
                    Err(e) => {
                        log::warn!("Invalid CDN wildcard pattern '{}': {}", pattern, e);
                    }
                }
            } else if pattern.contains('[') || pattern.contains('+')
                || pattern.contains('(') || pattern.contains('{') {
                // Treat as regex pattern directly
                match Regex::new(&pattern) {
                    Ok(regex) => {
                        self.regex_matches.push((regex, entry));
                        log::debug!("Added CDN regex pattern: {}", pattern);
                    }
                    Err(e) => {
                        log::warn!("Invalid CDN regex pattern '{}': {}", pattern, e);
                    }
                }
            } else {
                // Treat as exact match
                self.exact_matches.insert(pattern.to_lowercase(), entry);
                log::debug!("Added CDN exact match: {}", pattern);
            }
        }

        Ok(())
    }

    /// Convert wildcard pattern to regex
    /// e.g., "*.example.com" -> "^.*\\.example\\.com$"
    fn wildcard_to_regex(pattern: &str) -> String {
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
}

impl Database for CDNDatabase {
    fn name(&self) -> &str {
        &self.name
    }

    fn database_type(&self) -> DatabaseType {
        DatabaseType::CDN
    }

    fn supports_ipv4(&self) -> bool {
        false
    }

    fn supports_ipv6(&self) -> bool {
        false
    }

    fn supports_cdn(&self) -> bool {
        true
    }

    fn lookup_ip(&self, _ip: IpAddr) -> Result<Option<GeoLocation>> {
        // CDN database doesn't support IP lookup
        Ok(None)
    }

    fn lookup_cdn(&self, domain: &str) -> Result<Option<CdnProvider>> {
        if !self.loaded {
            return Err(NaliError::DatabaseNotLoaded("cdn".to_string()));
        }

        let domain_lower = domain.to_lowercase();

        // Try exact match first
        if let Some(entry) = self.exact_matches.get(&domain_lower) {
            return Ok(Some(CdnProvider {
                domain: domain.to_string(),
                provider: entry.name.clone(),
                description: entry.link.clone(),
            }));
        }

        // Try base domain matches
        let candidates = Self::extract_base_domain(&domain_lower);
        for candidate in &candidates {
            if let Some(entry) = self.exact_matches.get(candidate) {
                return Ok(Some(CdnProvider {
                    domain: domain.to_string(),
                    provider: entry.name.clone(),
                    description: entry.link.clone(),
                }));
            }
        }

        // Try regex matches
        if let Some(entry) = self.match_regex(&domain_lower) {
            return Ok(Some(CdnProvider {
                domain: domain.to_string(),
                provider: entry.name.clone(),
                description: entry.link.clone(),
            }));
        }

        // Not found
        Ok(None)
    }

    fn is_loaded(&self) -> bool {
        self.loaded
    }

    fn load_from_file(&mut self, file_path: &str) -> Result<()> {
        log::info!("Loading CDN database from: {}", file_path);

        let content = fs::read_to_string(file_path)
            .map_err(|e| NaliError::IoError(e))?;

        self.parse_yaml(&content)?;

        self.loaded = true;
        log::info!(
            "Successfully loaded CDN database: {} exact, {} regex patterns",
            self.exact_matches.len(),
            self.regex_matches.len()
        );

        Ok(())
    }
}

impl Default for CDNDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_base_domain() {
        let candidates = CDNDatabase::extract_base_domain("www.example.com");
        assert!(candidates.contains(&"www.example.com".to_string()));
        assert!(candidates.contains(&"example.com".to_string()));
    }

    #[test]
    fn test_parse_simple_yaml() {
        let yaml = r#"
cloudflare.com:
  name: Cloudflare
  link: https://www.cloudflare.com
"#;

        let mut db = CDNDatabase::new();
        db.parse_yaml(yaml).unwrap();
        assert_eq!(db.exact_matches.len(), 1);
    }

    #[test]
    fn test_lookup_exact_match() {
        let yaml = r#"
cloudflare.com:
  name: Cloudflare
"#;

        let mut db = CDNDatabase::new();
        db.parse_yaml(yaml).unwrap();
        db.loaded = true;

        let result = db.lookup_cdn("cloudflare.com").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().provider, "Cloudflare");
    }

    #[test]
    fn test_lookup_subdomain() {
        let yaml = r#"
example.com:
  name: Example CDN
"#;

        let mut db = CDNDatabase::new();
        db.parse_yaml(yaml).unwrap();
        db.loaded = true;

        let result = db.lookup_cdn("www.example.com").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().provider, "Example CDN");
    }

    #[test]
    fn test_lookup_regex() {
        let yaml = r#"
"[a-z]+\\.cdn\\.example\\.com":
  name: Example CDN Network
"#;

        let mut db = CDNDatabase::new();
        db.parse_yaml(yaml).unwrap();
        db.loaded = true;

        let result = db.lookup_cdn("test.cdn.example.com").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().provider, "Example CDN Network");
    }

    #[test]
    fn test_lookup_not_found() {
        let yaml = r#"
cloudflare.com:
  name: Cloudflare
"#;

        let mut db = CDNDatabase::new();
        db.parse_yaml(yaml).unwrap();
        db.loaded = true;

        let result = db.lookup_cdn("unknown.com").unwrap();
        assert!(result.is_none());
    }
}
