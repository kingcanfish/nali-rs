//! CDN database implementation
//!
//! This module implements support for CDN provider identification
//! based on domain names using YAML configuration.
//!
//! # Module Organization
//!
//! - `database`: Core CDNDatabase implementation
//! - `entry`: CDN entry types
//! - `matcher`: Domain matching utilities

mod database;
mod entry;
mod matcher;

// Re-export the main database struct and entry type
pub use database::CDNDatabase;
pub use entry::CdnEntry;
