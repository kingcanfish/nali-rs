//! Core types and error handling
//!
//! This module re-exports commonly used types and traits for the nali-rs crate.

pub use anyhow::{anyhow, Context, Result};
pub use log::{debug, error, info, warn};
pub use serde::{Deserialize, Serialize};
pub use std::net::IpAddr;
