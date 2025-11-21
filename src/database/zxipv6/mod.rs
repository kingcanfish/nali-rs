//! ZX IPv6 database implementation
//!
//! This module implements support for the ZX IPv6 database format,
//! which provides IPv6 geolocation information for Chinese networks.
//!
//! # Module Organization
//!
//! - `database`: Core ZXIPv6Database implementation
//! - `reader`: Binary format reader for parsing ZX IPv6 data
//! - `utils`: Utility functions for data conversion and validation

mod database;
mod reader;
mod utils;

// Re-export the main database struct
pub use database::ZXIPv6Database;
