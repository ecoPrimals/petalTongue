//! petalTongue configuration.

use serde::{Deserialize, Serialize};
use sourdough_core::config::CommonConfig;

/// Configuration for petalTongue.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct petalTongueConfig {
    /// Common configuration.
    #[serde(flatten)]
    pub common: CommonConfig,
    
    // TODO: Add petalTongue-specific configuration
}

impl Default for petalTongueConfig {
    fn default() -> Self {
        Self {
            common: CommonConfig {
                name: "petalTongue".to_string(),
                ..CommonConfig::default()
            },
        }
    }
}
