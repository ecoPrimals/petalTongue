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

pub mod config;
pub mod error;
pub mod types;
pub mod graph_engine;

use sourdough_core::{
    PrimalLifecycle, PrimalHealth, PrimalState, PrimalError,
    health::{HealthStatus, HealthReport},
};

/// petalTongue configuration.
pub use config::petalTongueConfig;

/// petalTongue errors.
pub use error::petalTongueError;

/// Visualization types
pub use types::*;

/// Graph engine (core topology representation)
pub use graph_engine::{GraphEngine, Node, Position, LayoutAlgorithm, GraphStats};

/// The petalTongue primal.
pub struct petalTongue {
    config: petalTongueConfig,
    state: PrimalState,
}

impl petalTongue {
    /// Create a new petalTongue instance.
    pub fn new(config: petalTongueConfig) -> Self {
        Self {
            config,
            state: PrimalState::Created,
        }
    }
}

impl PrimalLifecycle for petalTongue {
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

impl PrimalHealth for petalTongue {
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
