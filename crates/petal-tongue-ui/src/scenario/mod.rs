//! Scenario loading and management for benchTop demonstrations
//!
//! This module provides the core scenario system that allows petalTongue
//! to load and display pre-configured ecosystem topologies for:
//! - Demonstrations and benchmarking
//! - UI capability showcases
//! - Cross-device rendering tests
//! - Tutorial modes
//!
//! ## Module Structure
//!
//! - **`types`**: Core Scenario struct and main type definitions
//! - **`config`**: UI configuration (panels, animations, performance)
//! - **`ecosystem`**: Primal definitions, positions, and metrics
//! - **`sensory`**: Sensory capability configuration for adaptive rendering
//! - **`loader`**: Loading and validation logic
//! - **`convert`**: Type conversions to core PrimalInfo
//! - **`fixtures`**: Test fixtures (test-only)
//!
//! ## Architecture
//!
//! Scenarios are JSON files that define:
//! 1. **Ecosystem**: The primals, their positions, and relationships
//! 2. **UI Config**: Visual settings, panels, and animations
//! 3. **Sensory Config**: Required/optional capabilities for cross-device support
//! 4. **Metadata**: Name, version, mode, description
//!
//! ## Usage
//!
//! ```rust
//! use petal_tongue_ui::scenario::Scenario;
//!
//! // Load a scenario from JSON
//! let scenario = Scenario::load("scenarios/demo.json")?;
//!
//! // Validate against device capabilities
//! let caps = discover_sensory_capabilities();
//! scenario.validate_capabilities(&caps)?;
//!
//! // Determine UI complexity for this device
//! let complexity = scenario.determine_complexity(&caps);
//!
//! // Convert to graph data
//! let primals = scenario.to_primal_infos();
//! ```
//!
//! ## Design Philosophy
//!
//! - **Validation First**: All scenarios are validated on load
//! - **Cross-Device**: Sensory config enables graceful degradation
//! - **Type-Safe**: Strong typing with clear conversion boundaries
//! - **Modular**: Clean separation between config, data, and logic
//! - **Testable**: Comprehensive test fixtures and validation

pub mod config;
pub mod convert;
pub mod ecosystem;
pub mod loader;
pub mod sensory;
pub mod types;

// Re-export main types for convenience
pub use config::{
    AnimationConfig, CustomPanelConfig, FeatureFlags, PanelVisibility, PerformanceConfig, UiConfig,
};
pub use ecosystem::{Ecosystem, Position, PrimalDefinition, PrimalMetrics};
pub use sensory::{CapabilityRequirements, SensoryConfig};
pub use types::{NeuralApiConfig, Scenario, ScenarioProprioception};
