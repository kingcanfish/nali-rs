//! Entity parsing and processing module
//!
//! This module extracts entities (IP addresses and domains) from text
//! and enriches them with geolocation/CDN information.

pub mod parser;
pub mod types;
pub mod formatter;

pub use types::*;
