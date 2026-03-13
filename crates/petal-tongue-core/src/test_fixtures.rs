// SPDX-License-Identifier: AGPL-3.0-only
//! Test fixtures and constants
//!
//! Centralized location for all test data to avoid hardcoding throughout tests.
//!
//! **Gated**: This entire module is only compiled when `test` or `test-fixtures` feature
//! is enabled. Production builds (default) do NOT include this code.

use crate::{PrimalHealthStatus as HealthStatus, PrimalInfo};

/// Environment variable test helpers.
///
/// Thin wrappers around `temp_env` that maintain API compatibility.
/// Zero `unsafe` -- `temp_env` handles the Rust 2024 `set_var` safety
/// contract internally with proper mutex serialization.
///
/// **Important**: Do NOT nest `with_env_var` / `with_env_var_removed` calls --
/// `temp_env` uses a non-reentrant mutex and will deadlock. Use `with_env_vars`
/// for multiple vars, or run sequentially (not nested).
#[cfg(any(test, feature = "test-fixtures"))]
pub mod env_test_helpers {
    /// Temporarily set multiple env vars. `None` means remove.
    pub fn with_env_vars<F, R>(vars: &[(&str, Option<&str>)], f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let owned: Vec<(String, Option<String>)> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.map(ToString::to_string)))
            .collect();
        temp_env::with_vars(owned, f)
    }

    /// Temporarily set an env var for testing, restoring the original after.
    pub fn with_env_var<F, R>(key: &str, value: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        temp_env::with_var(key, Some(value), f)
    }

    /// Temporarily remove an env var for testing, restoring the original after.
    pub fn with_env_var_removed<F, R>(key: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        temp_env::with_var_unset(key, f)
    }

    /// Async version: temporarily set an env var for testing, restoring after.
    pub async fn with_env_var_async<F, Fut, R>(key: &str, value: &str, f: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        temp_env::async_with_vars(vec![(key.to_string(), Some(value.to_string()))], f()).await
    }

    /// Async version: temporarily remove an env var for testing, restoring after.
    pub async fn with_env_var_removed_async<F, Fut, R>(key: &str, f: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        temp_env::async_with_vars(vec![(key.to_string(), None::<String>)], f()).await
    }

    /// Async version: set multiple env vars at once.
    /// `None` values remove the variable.
    pub async fn with_env_vars_async<F, Fut, R>(vars: &[(&str, Option<&str>)], f: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let owned: Vec<(String, Option<String>)> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.map(ToString::to_string)))
            .collect();
        temp_env::async_with_vars(owned, f()).await
    }

    /// Async version: temporarily remove multiple env vars for testing.
    pub async fn with_env_vars_removed_async<F, Fut, R>(keys: &[&str], f: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let owned: Vec<(String, Option<String>)> =
            keys.iter().map(|k| (k.to_string(), None)).collect();
        temp_env::async_with_vars(owned, f()).await
    }
}

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
            id,
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
