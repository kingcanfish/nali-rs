//! CDN database entry types

use serde::{Deserialize, Serialize};

/// CDN database entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnEntry {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
}
