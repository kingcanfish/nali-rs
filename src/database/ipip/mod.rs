//! IPIP database implementation
//!
//! This module implements support for the IPIP database format.
//! IPIP is known for its balance between accuracy and performance,
//! supporting both IPv4 and IPv6 geolocation lookup.
//!
//! # Module Organization
//!
//! - `database`: Core IPIPDatabase implementation
//! - `header`: Database header structure and parsing
//! - `record`: IP range record structure
//! - `translation`: Translation tables for ID to string conversion

mod database;
mod header;
mod record;
mod translation;

// Re-export the main database struct
pub use database::IPIPDatabase;
