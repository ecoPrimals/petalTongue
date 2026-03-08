// SPDX-License-Identifier: AGPL-3.0-only
//! Test fixtures and constants
//!
//! Centralized location for all test data to avoid hardcoding throughout tests.

use crate::{PrimalHealthStatus as HealthStatus, PrimalInfo};

/// Test endpoint constants
pub mod endpoints {
    /// Mock `BiomeOS` endpoint for tests
    pub const MOCK_BIOMEOS: &str = "http://test-biomeos:3000";

    /// Mock primal endpoint base
    pub const MOCK_PRIMAL_BASE: &str = "http://test-primal";

    /// Generate mock primal endpoint with ID
    #[must_use]
    pub fn primal_endpoint(id: u32) -> String {
        format!("{}:{}", MOCK_PRIMAL_BASE, 8000 + id)
    }
}

/// Test primal info builders
pub mod primals {
    use super::{HealthStatus, PrimalInfo};

    /// Create a test primal with sensible defaults
    #[must_use]
    pub fn test_primal(id: &str) -> PrimalInfo {
        PrimalInfo::new(
            id.to_string(),
            format!("Test Primal {id}"),
            "TestPrimal".to_string(),
            super::endpoints::primal_endpoint(id.parse().unwrap_or(0)),
            vec!["test.capability".to_string()],
            HealthStatus::Healthy,
            0, // Unix timestamp
        )
    }

    /// Create a test primal with specific type
    #[must_use]
    pub fn test_primal_with_type(id: &str, primal_type: &str) -> PrimalInfo {
        PrimalInfo::new(
            id.to_string(),
            format!("Test Primal {id}"),
            primal_type.to_string(),
            super::endpoints::primal_endpoint(id.parse().unwrap_or(0)),
            vec![],
            HealthStatus::Healthy,
            0, // Unix timestamp
        )
    }

    /// Create a test primal with specific health
    #[must_use]
    pub fn test_primal_with_health(id: &str, health: HealthStatus) -> PrimalInfo {
        let mut primal = test_primal(id);
        primal.health = health;
        primal
    }
}
