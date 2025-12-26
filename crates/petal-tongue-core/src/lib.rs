//! # petalTongue
//!
//! Universal UI and Visualization System
//!
//! ## Overview
//!
//! petalTongue is part of the ecoPrimals ecosystem.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use petal_tongue_core::petalTongue;
//!
//! let primal = petalTongue::new(config).await?;
//! primal.start().await?;
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod capabilities;
pub mod config;
pub mod error;
pub mod graph_engine;
pub mod types;

use sourdough_core::{
    PrimalError, PrimalHealth, PrimalLifecycle, PrimalState,
    health::{HealthReport, HealthStatus},
};

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

        // TODO: Initialize resources

        self.state = PrimalState::Running;
        tracing::info!("petalTongue running");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PrimalError> {
        self.state = PrimalState::Stopping;
        tracing::info!("petalTongue stopping...");

        // TODO: Clean up resources

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
