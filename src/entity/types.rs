//! Entity types and structures

use crate::database::{GeoLocation, CdnProvider};
use std::net::IpAddr;

/// Entity type classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityType {
    /// IPv4 address
    IPv4,
    /// IPv6 address
    IPv6,
    /// Domain name
    Domain,
    /// Plain text (not an entity)
    Plain,
}

/// An entity extracted from text
#[derive(Debug, Clone)]
pub struct Entity {
    /// Position in original text (start, end)
    pub location: (usize, usize),

    /// Entity type
    pub entity_type: EntityType,

    /// Original text
    pub text: String,

    /// Geolocation information (for IP entities)
    pub geo_info: Option<GeoLocation>,

    /// CDN provider information (for domain entities)
    pub cdn_info: Option<CdnProvider>,

    /// Source database name
    pub source: Option<String>,
}

impl Entity {
    /// Create a new plain text entity
    pub fn plain(start: usize, end: usize, text: String) -> Self {
        Entity {
            location: (start, end),
            entity_type: EntityType::Plain,
            text,
            geo_info: None,
            cdn_info: None,
            source: None,
        }
    }

    /// Create a new IPv4 entity
    pub fn ipv4(start: usize, end: usize, text: String) -> Self {
        Entity {
            location: (start, end),
            entity_type: EntityType::IPv4,
            text,
            geo_info: None,
            cdn_info: None,
            source: None,
        }
    }

    /// Create a new IPv6 entity
    pub fn ipv6(start: usize, end: usize, text: String) -> Self {
        Entity {
            location: (start, end),
            entity_type: EntityType::IPv6,
            text,
            geo_info: None,
            cdn_info: None,
            source: None,
        }
    }

    /// Create a new domain entity
    pub fn domain(start: usize, end: usize, text: String) -> Self {
        Entity {
            location: (start, end),
            entity_type: EntityType::Domain,
            text,
            geo_info: None,
            cdn_info: None,
            source: None,
        }
    }

    /// Check if this entity is an IP address
    pub fn is_ip(&self) -> bool {
        matches!(self.entity_type, EntityType::IPv4 | EntityType::IPv6)
    }

    /// Check if this entity is a domain
    pub fn is_domain(&self) -> bool {
        matches!(self.entity_type, EntityType::Domain)
    }

    /// Get parsed IP address if this is an IP entity
    pub fn as_ip(&self) -> Option<IpAddr> {
        if self.is_ip() {
            self.text.parse().ok()
        } else {
            None
        }
    }

    /// Check if entity has geolocation information
    pub fn has_geo_info(&self) -> bool {
        self.geo_info.is_some()
    }

    /// Check if entity has CDN information
    pub fn has_cdn_info(&self) -> bool {
        self.cdn_info.is_some()
    }
}

/// Collection of entities extracted from text
#[derive(Debug, Clone)]
pub struct Entities {
    pub entities: Vec<Entity>,
}

impl Entities {
    /// Create an empty collection
    pub fn new() -> Self {
        Entities {
            entities: Vec::new(),
        }
    }

    /// Add an entity to the collection
    pub fn push(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    /// Sort entities by their position in text
    pub fn sort_by_position(&mut self) {
        self.entities.sort_by_key(|e| e.location.0);
    }

    /// Remove overlapping entities (keep first occurrence)
    pub fn remove_overlaps(&mut self) {
        self.sort_by_position();

        let mut i = 0;
        while i < self.entities.len() {
            let mut j = i + 1;
            while j < self.entities.len() {
                // Check if entities overlap
                let (_start_i, end_i) = self.entities[i].location;
                let (start_j, _end_j) = self.entities[j].location;

                if start_j < end_i {
                    // Overlaps, remove the second one
                    self.entities.remove(j);
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
    }

    /// Get all IP entities
    pub fn ips(&self) -> Vec<&Entity> {
        self.entities.iter().filter(|e| e.is_ip()).collect()
    }

    /// Get all domain entities
    pub fn domains(&self) -> Vec<&Entity> {
        self.entities.iter().filter(|e| e.is_domain()).collect()
    }

    /// Count of all entities
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Check if collection is empty
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

impl Default for Entities {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let entity = Entity::ipv4(0, 10, "192.168.1.1".to_string());
        assert_eq!(entity.entity_type, EntityType::IPv4);
        assert!(entity.is_ip());
        assert!(!entity.is_domain());
    }

    #[test]
    fn test_entity_as_ip() {
        let entity = Entity::ipv4(0, 10, "192.168.1.1".to_string());
        let ip = entity.as_ip();
        assert!(ip.is_some());
        assert_eq!(ip.unwrap().to_string(), "192.168.1.1");
    }

    #[test]
    fn test_entities_sort() {
        let mut entities = Entities::new();
        entities.push(Entity::ipv4(10, 20, "8.8.8.8".to_string()));
        entities.push(Entity::ipv4(0, 10, "1.1.1.1".to_string()));

        entities.sort_by_position();

        assert_eq!(entities.entities[0].text, "1.1.1.1");
        assert_eq!(entities.entities[1].text, "8.8.8.8");
    }

    #[test]
    fn test_remove_overlaps() {
        let mut entities = Entities::new();
        entities.push(Entity::ipv4(0, 10, "192.168.1.1".to_string()));
        entities.push(Entity::plain(5, 15, "overlap".to_string()));

        entities.remove_overlaps();

        assert_eq!(entities.len(), 1);
        assert_eq!(entities.entities[0].text, "192.168.1.1");
    }
}
