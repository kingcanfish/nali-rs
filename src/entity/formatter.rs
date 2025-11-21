//! Entity formatter - formats entities for output

use crate::entity::types::{Entities, Entity, EntityType};
use std::fmt::Write as FmtWrite;

#[cfg(feature = "colored-output")]
use colored::Colorize;

/// Output format
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    /// Plain text with inline annotations
    Text,
    /// JSON format
    Json,
    /// Colored text (if feature enabled)
    Colored,
}

/// Color type for formatted output
#[cfg(feature = "colored-output")]
#[derive(Debug, Clone, Copy)]
enum ColorType {
    Green,
    Cyan,
}

/// Apply color to text if enabled
fn apply_color(text: &str, use_color: bool, color_type: ColorType) -> String {
    if !use_color {
        return text.to_string();
    }

    #[cfg(feature = "colored-output")]
    {
        match color_type {
            ColorType::Green => text.green().to_string(),
            ColorType::Cyan => text.cyan().to_string(),
        }
    }

    #[cfg(not(feature = "colored-output"))]
    {
        text.to_string()
    }
}

/// Format entities as text with inline geolocation information
pub fn format_text(entities: &Entities, use_color: bool) -> String {
    let mut result = String::new();

    for entity in &entities.entities {
        // Add the original text
        result.push_str(&entity.text);

        // Add geolocation info for IP entities
        if entity.has_geo_info()
            && let Some(ref geo) = entity.geo_info {
                let info = format_geo_info(geo);
                #[cfg(feature = "colored-output")]
                let formatted = apply_color(&info, use_color, ColorType::Green);
                #[cfg(not(feature = "colored-output"))]
                let formatted = info.clone();

                result.push_str(&format!(" [{}] ", formatted));
            }

        // Add CDN info for domain entities
        if entity.has_cdn_info()
            && let Some(ref cdn) = entity.cdn_info {
                let info = cdn.provider.to_string();
                #[cfg(feature = "colored-output")]
                let formatted = apply_color(&info, use_color, ColorType::Cyan);
                #[cfg(not(feature = "colored-output"))]
                let formatted = info.clone();

                result.push_str(&format!(" [{}] ", formatted));
            }
    }

    result
}

/// Format geolocation information as a compact string
fn format_geo_info(geo: &crate::database::GeoLocation) -> String {
    let mut parts = Vec::with_capacity(4);

    if let Some(ref country) = geo.country {
        parts.push(country.as_str());
    }

    // Use filter to avoid duplicate checking
    if let Some(ref region) = geo.region
        && geo.country.as_ref().is_none_or(|c| c != region) {
            parts.push(region.as_str());
        }

    if let Some(ref city) = geo.city
        && geo.region.as_ref().is_none_or(|r| r != city) {
            parts.push(city.as_str());
        }

    if let Some(ref isp) = geo.isp {
        parts.push(isp.as_str());
    }

    parts.join(" ")
}

/// Format geolocation information as a compact string (public API)
pub fn format_geo_info_compact(geo: &crate::database::GeoLocation) -> String {
    let result = format_geo_info(geo);
    if result.is_empty() {
        "[Unknown]".to_string()
    } else {
        result
    }
}

/// Format entities as JSON
pub fn format_json(entities: &Entities) -> Result<String, serde_json::Error> {
    use serde_json::json;

    let items: Vec<_> = entities
        .entities
        .iter()
        .filter(|e| e.entity_type != EntityType::Plain)
        .map(|e| {
            json!({
                "text": e.text,
                "type": format!("{:?}", e.entity_type),
                "position": {
                    "start": e.location.0,
                    "end": e.location.1,
                },
                "geo_info": e.geo_info,
                "cdn_info": e.cdn_info,
                "source": e.source,
            })
        })
        .collect();

    serde_json::to_string_pretty(&json!({
        "entities": items
    }))
}

/// Format single entity information
#[allow(dead_code)]
pub fn format_entity(entity: &Entity) -> String {
    let mut result = String::new();

    write!(&mut result, "{}", entity.text).unwrap();

    if let Some(ref geo) = entity.geo_info {
        write!(&mut result, " -> {}", format_geo_info(geo)).unwrap();
    }

    if let Some(ref cdn) = entity.cdn_info {
        write!(&mut result, " -> {}", cdn.provider).unwrap();
    }

    if let Some(ref source) = entity.source {
        write!(&mut result, " ({})", source).unwrap();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::types::Entities;
    use crate::database::GeoLocation;
    use std::net::IpAddr;

    #[test]
    fn test_format_text_plain() {
        let mut entities = Entities::new();
        entities.push(Entity::plain(0, 5, "Hello".to_string()));

        let formatted = format_text(&entities, false);
        assert_eq!(formatted, "Hello");
    }

    #[test]
    fn test_format_text_with_geo() {
        let mut entities = Entities::new();
        let mut entity = Entity::ipv4(0, 9, "8.8.8.8".to_string());

        entity.geo_info = Some(GeoLocation {
            ip: "8.8.8.8".parse::<IpAddr>().unwrap(),
            country: Some("美国".to_string()),
            region: Some("加利福尼亚".to_string()),
            city: Some("山景城".to_string()),
            isp: Some("Google".to_string()),
            country_code: None,
            timezone: None,
            latitude: None,
            longitude: None,
        });

        entities.push(entity);

        let formatted = format_text(&entities, false);
        assert!(formatted.contains("8.8.8.8"));
        assert!(formatted.contains("["));
        assert!(formatted.contains("美国"));
    }

    #[test]
    fn test_format_json() {
        let mut entities = Entities::new();
        entities.push(Entity::ipv4(0, 9, "8.8.8.8".to_string()));

        let json_result = format_json(&entities);
        assert!(json_result.is_ok());

        let json = json_result.unwrap();
        assert!(json.contains("entities"));
        assert!(json.contains("8.8.8.8"));
    }
}
