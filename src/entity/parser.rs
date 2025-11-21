//! Entity parser - extracts entities from text

use crate::entity::types::{Entities, Entity, EntityType};
use crate::regex::{find_ipv4, find_ipv6, find_domains};

/// Parse a line of text and extract all entities (IP addresses and domains)
///
/// This function searches for IPv4, IPv6 addresses, and domain names in the input text.
/// Overlapping entities are removed, keeping the first occurrence.
///
/// # Arguments
///
/// * `text` - The input text to parse
///
/// # Returns
///
/// An `Entities` collection containing all found entities, sorted by position
///
/// # Performance
///
/// Time complexity: O(n) where n is the length of the text
///
/// # Example
///
/// ```
/// use nali_rs::entity::parser::parse_line;
///
/// let text = "Server IP: 192.168.1.1";
/// let entities = parse_line(text);
/// assert_eq!(entities.len(), 1);
/// ```
pub fn parse_line(text: &str) -> Entities {
    let mut entities = Entities::new();

    // Find all IPv4 addresses
    for (start, end, ipv4_text) in find_ipv4(text) {
        entities.push(Entity::ipv4(start, end, ipv4_text));
    }

    // Find all IPv6 addresses
    for (start, end, ipv6_text) in find_ipv6(text) {
        entities.push(Entity::ipv6(start, end, ipv6_text));
    }

    // Find all domains
    for (start, end, domain_text) in find_domains(text) {
        // Skip if it's actually part of an IPv4 address
        // (domain regex might match some IP patterns)
        if !entities.entities.iter().any(|e| {
            e.entity_type == EntityType::IPv4
                && e.location.0 <= start
                && e.location.1 >= end
        }) {
            entities.push(Entity::domain(start, end, domain_text));
        }
    }

    // Remove overlapping entities
    entities.remove_overlaps();

    // Sort by position
    entities.sort_by_position();

    entities
}

/// Parse multiple lines of text
///
/// Convenience function that calls `parse_line` for each line.
///
/// # Arguments
///
/// * `lines` - A slice of strings to parse
///
/// # Returns
///
/// A vector of `Entities` collections, one for each line
pub fn parse_lines(lines: &[String]) -> Vec<Entities> {
    lines.iter().map(|line| parse_line(line)).collect()
}

/// Build a complete entity list with plain text segments
///
/// This fills in the gaps between extracted entities with plain text segments,
/// so that the original text can be reconstructed with enriched information.
pub fn build_complete_entities(text: &str, mut entities: Entities) -> Entities {
    if entities.is_empty() {
        // No entities found, return the whole text as plain
        let mut result = Entities::new();
        result.push(Entity::plain(0, text.len(), text.to_string()));
        return result;
    }

    entities.sort_by_position();

    let mut complete = Entities::new();
    let mut last_pos = 0;

    for entity in entities.entities {
        let (start, end) = entity.location;

        // Add plain text before this entity
        if start > last_pos {
            let plain_text = &text[last_pos..start];
            complete.push(Entity::plain(last_pos, start, plain_text.to_string()));
        }

        // Add the entity
        complete.push(entity);
        last_pos = end;
    }

    // Add remaining plain text
    if last_pos < text.len() {
        let plain_text = &text[last_pos..];
        complete.push(Entity::plain(
            last_pos,
            text.len(),
            plain_text.to_string(),
        ));
    }

    complete
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_line_ipv4() {
        let text = "Server IP: 192.168.1.1";
        let entities = parse_line(text);

        assert_eq!(entities.len(), 1);
        assert_eq!(entities.entities[0].entity_type, EntityType::IPv4);
        assert_eq!(entities.entities[0].text, "192.168.1.1");
    }

    #[test]
    fn test_parse_line_multiple() {
        let text = "IP 8.8.8.8 at google.com";
        let entities = parse_line(text);

        assert!(entities.len() >= 2);

        let ips = entities.ips();
        let domains = entities.domains();

        assert!(!ips.is_empty());
        assert!(!domains.is_empty());
    }

    #[test]
    fn test_build_complete_entities() {
        let text = "Server: 1.2.3.4 ok";
        let entities = parse_line(text);
        let complete = build_complete_entities(text, entities);

        // Should have plain + ip + plain
        assert_eq!(complete.len(), 3);
        assert_eq!(complete.entities[0].text, "Server: ");
        assert_eq!(complete.entities[1].text, "1.2.3.4");
        assert_eq!(complete.entities[2].text, " ok");
    }

    #[test]
    fn test_no_entities() {
        let text = "No IPs or domains here";
        let entities = parse_line(text);
        let complete = build_complete_entities(text, entities);

        assert_eq!(complete.len(), 1);
        assert_eq!(complete.entities[0].entity_type, EntityType::Plain);
        assert_eq!(complete.entities[0].text, text);
    }
}
