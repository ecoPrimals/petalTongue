// SPDX-License-Identifier: AGPL-3.0-or-later
//! Serde types and graph metadata for templates and persistence.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Graph metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphMetadata {
    /// Tags for categorization
    pub tags: Vec<String>,

    /// Author (if saved as template)
    pub author: Option<String>,

    /// Creation timestamp
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Last modified timestamp
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Template ID (if loaded from template)
    pub template_id: Option<String>,

    /// Custom metadata (extensible)
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for GraphMetadata {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            author: None,
            created_at: Some(chrono::Utc::now()),
            modified_at: Some(chrono::Utc::now()),
            template_id: None,
            custom: HashMap::new(),
        }
    }
}
