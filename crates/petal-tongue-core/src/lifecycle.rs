//! Primal lifecycle traits.
//!
//! These traits define the lifecycle and health management for petalTongue.
//! Previously sourced from sourdough-core, now self-contained for independence.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Primal state in its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalState {
    /// Primal has been created but not started
    Created,
    /// Primal is starting up
    Starting,
    /// Primal is running normally
    Running,
    /// Primal is stopping
    Stopping,
    /// Primal has stopped
    Stopped,
    /// Primal has encountered a fatal error
    Failed,
}

impl PrimalState {
    /// Check if primal is in a running state.
    #[must_use]
    pub const fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }
}

impl fmt::Display for PrimalState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Starting => write!(f, "starting"),
            Self::Running => write!(f, "running"),
            Self::Stopping => write!(f, "stopping"),
            Self::Stopped => write!(f, "stopped"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Health status of a primal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Primal is healthy
    Healthy,
    /// Primal is unhealthy with a reason
    Unhealthy { reason: String },
}

impl HealthStatus {
    /// Check if status is healthy.
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }
}

/// Health report for a primal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Name of the primal
    pub name: String,
    /// Version of the primal
    pub version: String,
    /// Current health status
    pub status: HealthStatus,
}

impl HealthReport {
    /// Create a new health report.
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            status: HealthStatus::Healthy,
        }
    }

    /// Set the health status.
    #[must_use]
    pub fn with_status(mut self, status: HealthStatus) -> Self {
        self.status = status;
        self
    }
}

/// Primal error type.
#[derive(Debug, thiserror::Error)]
pub enum PrimalError {
    /// Configuration error
    #[error("configuration error: {0}")]
    Config(String),
    
    /// Lifecycle error
    #[error("lifecycle error: {0}")]
    Lifecycle(String),
    
    /// Health check error
    #[error("health check error: {0}")]
    Health(String),
    
    /// Generic error
    #[error("{0}")]
    Other(String),
}

/// Trait for primal lifecycle management.
pub trait PrimalLifecycle {
    /// Get current primal state.
    fn state(&self) -> PrimalState;

    /// Start the primal.
    ///
    /// # Errors
    ///
    /// Returns error if startup fails.
    fn start(&mut self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send;

    /// Stop the primal.
    ///
    /// # Errors
    ///
    /// Returns error if shutdown fails.
    fn stop(&mut self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send;
}

/// Trait for primal health management.
pub trait PrimalHealth {
    /// Get current health status.
    fn health_status(&self) -> HealthStatus;

    /// Perform health check.
    ///
    /// # Errors
    ///
    /// Returns error if health check fails.
    fn health_check(&self) -> impl std::future::Future<Output = Result<HealthReport, PrimalError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_state() {
        assert!(PrimalState::Running.is_running());
        assert!(!PrimalState::Stopped.is_running());
        assert_eq!(PrimalState::Running.to_string(), "running");
    }

    #[test]
    fn test_health_status() {
        let healthy = HealthStatus::Healthy;
        assert!(healthy.is_healthy());

        let unhealthy = HealthStatus::Unhealthy {
            reason: "test".to_string(),
        };
        assert!(!unhealthy.is_healthy());
    }

    #[test]
    fn test_health_report() {
        let report = HealthReport::new("test", "1.0.0")
            .with_status(HealthStatus::Healthy);
        
        assert_eq!(report.name, "test");
        assert_eq!(report.version, "1.0.0");
        assert!(report.status.is_healthy());
    }
}

