// SPDX-License-Identifier: AGPL-3.0-only
//! Test fixtures and constants
//!
//! Centralized location for all test data to avoid hardcoding throughout tests.

use crate::{PrimalHealthStatus as HealthStatus, PrimalInfo};

/// Environment variable test helpers
///
/// Centralizes unsafe env var manipulation into ONE well-documented location
/// with proper safety invariants (mutex serialization).
///
/// **Important**: Do NOT nest `with_env_var` / `with_env_var_removed` calls -
/// they use a non-reentrant mutex and will deadlock. Use `with_env_vars` for
/// multiple vars, or run sequentially (not nested).
#[cfg(any(test, feature = "test-fixtures"))]
pub mod env_test_helpers {
    use std::sync::{Mutex, PoisonError};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Temporarily set multiple env vars. `None` means remove.
    /// Use this instead of nesting to avoid deadlock.
    pub fn with_env_vars<F, R>(vars: &[(&str, Option<&str>)], f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = ENV_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let originals: Vec<(String, Option<String>)> = vars
            .iter()
            .map(|(k, _)| (k.to_string(), std::env::var(k).ok()))
            .collect();
        for (key, val) in vars {
            match val {
                Some(v) => unsafe { std::env::set_var(key, v) },
                None => unsafe { std::env::remove_var(key) },
            }
        }
        let result = f();
        for (i, (key, _)) in vars.iter().enumerate() {
            if let Some((_, orig)) = originals.get(i) {
                match orig {
                    Some(v) => unsafe { std::env::set_var(key, v) },
                    None => unsafe { std::env::remove_var(key) },
                }
            }
        }
        result
    }

    /// Temporarily set an env var for testing, restoring the original after.
    pub fn with_env_var<F, R>(key: &str, value: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        with_env_vars(&[(key, Some(value))], f)
    }

    /// Temporarily remove an env var for testing, restoring the original after.
    pub fn with_env_var_removed<F, R>(key: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        with_env_vars(&[(key, None)], f)
    }

    /// Async version: temporarily set an env var for testing, restoring after.
    pub async fn with_env_var_async<F, Fut, R>(key: &str, value: &str, f: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let original = {
            let _guard = ENV_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
            let original = std::env::var(key).ok();
            unsafe { std::env::set_var(key, value) };
            original
        };
        let result = f().await;
        let _guard = ENV_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        match original {
            Some(val) => unsafe { std::env::set_var(key, &val) },
            None => unsafe { std::env::remove_var(key) },
        }
        result
    }

    /// Async version: temporarily remove an env var for testing, restoring after.
    pub async fn with_env_var_removed_async<F, Fut, R>(key: &str, f: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let original = {
            let _guard = ENV_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
            let original = std::env::var(key).ok();
            unsafe { std::env::remove_var(key) };
            original
        };
        let result = f().await;
        if let Some(val) = original {
            let _guard = ENV_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
            unsafe { std::env::set_var(key, &val) };
        }
        result
    }

    /// Async version: set multiple env vars at once, avoiding nested deadlocks.
    /// `None` values remove the variable.
    pub async fn with_env_vars_async<F, Fut, R>(vars: &[(&str, Option<&str>)], f: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let originals: Vec<(String, Option<String>)> = {
            let _guard = ENV_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
            let originals: Vec<(String, Option<String>)> = vars
                .iter()
                .map(|(k, _)| (k.to_string(), std::env::var(k).ok()))
                .collect();
            for (key, val) in vars {
                match val {
                    Some(v) => unsafe { std::env::set_var(key, v) },
                    None => unsafe { std::env::remove_var(key) },
                }
            }
            originals
        };
        let result = f().await;
        {
            let _guard = ENV_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
            for (key, orig) in &originals {
                match orig {
                    Some(v) => unsafe { std::env::set_var(key, v) },
                    None => unsafe { std::env::remove_var(key) },
                }
            }
        }
        result
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
