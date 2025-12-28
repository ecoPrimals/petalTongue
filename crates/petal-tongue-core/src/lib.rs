//! petalTongue Core
//!
//! Core graph engine and types for multi-modal primal visualization.
//!
//! # Architecture
//!
//! - **Zero hardcoding** - All configuration is environment-driven
//! - **Capability-based** - Runtime discovery, no assumptions about primal names
//! - **Modality-agnostic** - Core knows nothing about rendering
//! - **Type-safe** - Strong typing throughout
//! - **Self-contained** - No external primal dependencies, only self-knowledge

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod capabilities;
pub mod common_config;
pub mod config;
#[cfg(test)]
mod config_tests;
pub mod error;
#[cfg(test)]
mod error_tests;
pub mod graph_engine;
pub mod lifecycle;
pub mod primal_types;
pub mod types;
#[cfg(test)]
mod types_tests;

// Test fixtures available for this and dependent crates
#[cfg(any(test, feature = "test-fixtures"))]
pub mod test_fixtures;

// Re-export lifecycle traits and types
pub use lifecycle::{
    HealthReport, HealthStatus, PrimalError, PrimalHealth, PrimalLifecycle, PrimalState,
};

// Re-export common config
pub use common_config::CommonConfig;

/// petalTongue configuration.
pub use config::PetalTongueConfig;

/// petalTongue errors.
pub use error::PetalTongueError;

/// Visualization types
pub use types::*;

/// Graph engine (core topology representation)
pub use graph_engine::{GraphEngine, GraphStats, LayoutAlgorithm};

/// Modality capability detection
pub use capabilities::{CapabilityDetector, Modality, ModalityCapability, ModalityStatus};

/// Capability-based primal type system
pub use primal_types::{PrimalCapabilities, capability_categories};

/// The petalTongue primal.
pub struct PetalTongue {
    config: PetalTongueConfig,
    state: PrimalState,
}

impl PetalTongue {
    /// Create a new petalTongue instance.
    #[must_use]
    pub fn new(config: PetalTongueConfig) -> Self {
        Self {
            config,
            state: PrimalState::Created,
        }
    }

    /// Get reference to configuration.
    #[must_use]
    pub const fn config(&self) -> &PetalTongueConfig {
        &self.config
    }
}

impl PrimalLifecycle for PetalTongue {
    fn state(&self) -> PrimalState {
        self.state
    }

    async fn start(&mut self) -> Result<(), PrimalError> {
        self.state = PrimalState::Starting;
        tracing::info!("petalTongue starting...");

        // Resources are initialized lazily by the UI framework (egui)
        // No explicit initialization needed here

        self.state = PrimalState::Running;
        tracing::info!("petalTongue running");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PrimalError> {
        self.state = PrimalState::Stopping;
        tracing::info!("petalTongue stopping...");

        // Resources are cleaned up automatically by Drop implementations
        // No explicit cleanup needed here

        self.state = PrimalState::Stopped;
        tracing::info!("petalTongue stopped");
        Ok(())
    }
}

impl PrimalHealth for PetalTongue {
    fn health_status(&self) -> HealthStatus {
        if self.state.is_running() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy {
                reason: format!("state: {}", self.state),
            }
        }
    }

    async fn health_check(&self) -> Result<HealthReport, PrimalError> {
        Ok(HealthReport::new("petalTongue", env!("CARGO_PKG_VERSION"))
            .with_status(self.health_status()))
    }
}
