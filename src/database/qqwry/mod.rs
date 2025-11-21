//! QQwry database implementation
//!
//! This module implements support for the QQwry IPv4 database format,
//! which is the most commonly used Chinese IP geolocation database.
//!
//! # Module Organization
//!
//! - `database`: Core QQwryDatabase implementation
//! - `reader`: Binary format reader for parsing QQwry data
//! - `utils`: Utility functions for data conversion

mod database;
mod reader;
mod utils;

// Re-export the main database struct
pub use database::QQwryDatabase;
