// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
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
//!
//! # New: Universal Rendering System (Phase 2026)
//!
//! "A graphical interface is simply the interconnection of information
//!  and how it is represented."
//!
//! - **Awakening Experience**: Default touchpoint (flower opening to sunrise)
//! - **Modality System**: Multiple representations (audio, visual, text)
//! - **Event Coordination**: Synchronize across modalities
//! - **Compute Integration**: Optional GPU acceleration (Toadstool)

#![warn(missing_docs)]

pub mod adaptive_rendering; // Adaptive rendering for multi-device support
pub mod biomeos_discovery; // biomeOS discovery backend
pub mod capabilities;
pub mod capability_discovery; // NEW: Capability-based discovery (TRUE PRIMAL)
pub mod capability_names;
pub mod channel; // SAME DAVE channel model (afferent/efferent pathways)
pub mod common_config;
pub mod config;
pub mod config_system; // NEW: Platform-agnostic configuration (XDG-compliant)
#[cfg(test)]
mod config_tests;
pub mod constants; // Centralized constants (self-knowledge only)
pub mod data_channel; // DataBinding and ThresholdRange (universal visualization)
pub mod dynamic_schema; // Dynamic schema system for live evolution
pub mod error;
#[cfg(test)]
mod error_tests;
pub mod graph_builder; // SAME DAVE proprioception data (Neural API)
pub mod graph_engine;
pub mod graph_validation; // Graph validation (cycle detection, dependencies)
pub mod instance; // Instance management
#[cfg(test)]
mod lib_tests;
pub mod lifecycle;
pub mod metrics; // System metrics (CPU, memory, Neural API stats)
pub mod platform_dirs; // Pure Rust directory resolution (zero deps!)
pub mod primal_types;
pub mod property; // Generic property system
pub mod proprioception;
pub mod scenario_builder; // ScenarioBuilder trait for springs and primals
pub mod scenario_loader; // healthSpring-style scenario JSON loader
pub mod scenarios; // Scenario builders for unintegrated springs (airSpring, groundSpring)
pub mod session; // Session state persistence (Phase 2)
pub mod shader_lineage; // Cross-spring shader lineage tracking and visualization
pub mod spring_adapter; // Universal spring data adapter (multi-format normalization)
pub mod state_sync; // State synchronization across devices
pub mod system_info; // System information utilities (safe FFI wrappers)
pub mod telemetry_adapter; // JSONL telemetry adapter (hotSpring ingestion)
pub mod types;
#[cfg(test)]
mod types_tests;

// NEW: Universal Rendering System
pub mod awakening; // Awakening experience (default touchpoint)
pub mod awakening_coordinator; // Timeline coordination for awakening
pub mod capability_taxonomy; // biomeOS capability taxonomy
pub mod compute; // Compute provider system (optional GPU)
pub mod engine; // Universal rendering engine
pub mod event; // Event bus (multi-modal coordination)
pub mod frame_introspection; // Content-level self-awareness (what each frame contains)
pub mod interaction; // Interaction engine (semantic intents, perspectives, inverse pipeline)
pub mod modality; // Modality system (trait and registry)
pub mod rendering_awareness; // Bidirectional UUI awareness (motor + sensory)
pub mod sensor; // Sensor abstraction layer
pub mod toadstool_compute; // Toadstool GPU compute integration
pub mod uui_glossary; // Universal User Interface glossary (canonical terminology)

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

/// Generic property system (ecosystem-agnostic)
pub use property::{Properties, PropertyValue};

/// Graph engine (core topology representation)
pub use graph_engine::{GraphEngine, GraphStats, LayoutAlgorithm};

/// Modality capability detection
pub use capabilities::{CapabilityDetector, Modality, ModalityCapability, ModalityStatus};

/// Capability-based primal type system
pub use primal_types::{PrimalCapabilities, capability_categories};

/// Proprioception data (SAME DAVE neuroanatomy: afferent/efferent channel snapshots)
pub use proprioception::{
    HealthData, HealthStatus as ProprioceptionHealthStatus, MotorData, ProprioceptionData,
    SelfAwarenessData, SensoryData,
};

/// System metrics (CPU, memory, Neural API statistics)
pub use metrics::{
    CpuHistory, MemoryHistory, NeuralApiMetrics, SystemMetrics, SystemResourceMetrics,
    ThresholdLevel,
};

/// Graph builder types (Visual graph construction - Neural API Phase 4)
pub use graph_builder::{
    EdgeType, GraphEdge, GraphLayout, GraphNode, NodeType, NodeVisualState, Vec2, VisualGraph,
};

/// Graph validation types (Cycle detection, dependency resolution)
pub use graph_validation::{GraphValidator, ValidationIssue, ValidationResult};

/// Data binding and threshold types (universal visualization)
pub use data_channel::{DataBinding, ThresholdRange};

/// Scenario builder trait (springs and primals produce visualization data)
pub use scenario_builder::{ScenarioBuilder, ScenarioMetadata, VisualizationScene};

/// Loaded scenario from healthSpring-style JSON
pub use scenario_loader::LoadedScenario;

/// Domain scenario builders (airSpring, groundSpring)
pub use scenarios::{
    AirSpringCropCoefficientScenario, AirSpringDroughtIndexScenario, AirSpringET0Scenario,
    AirSpringRichardsPDEScenario, GroundSpringAndersonLocalizationScenario,
    GroundSpringSeismicScenario, GroundSpringSensorDriftScenario,
    GroundSpringSpectralReconstructionScenario,
};

/// JSONL telemetry adapter (hotSpring ingestion → `DataBinding::TimeSeries`)
pub use telemetry_adapter::TelemetryAdapter;

/// Dynamic schema system (Live evolution, no recompilation)
pub use dynamic_schema::{
    DynamicData, DynamicValue, MigrationRegistry, SchemaMigration, SchemaVersion,
};

/// Adaptive rendering (Multi-device support)
pub use adaptive_rendering::{
    AdaptiveRenderer, DeviceType, HapticPrecision, InputMethod, PerformanceTier,
    RenderingCapabilities, RenderingModality, UIComplexity,
};

/// Cross-spring shader lineage tracking
pub use shader_lineage::{
    ShaderDelegation, ShaderLineage, ShaderLineageNode, ShaderLineageScenario,
    ShaderValidationStatus,
};

/// Universal spring data adapter (multi-format → `DataBinding` normalization)
pub use spring_adapter::{
    GameChannelType, SpringAdapterError, SpringDataAdapter, SpringPayloadFormat,
};

/// State synchronization (Cross-device state)
pub use state_sync::{DeviceState, LocalStatePersistence, StatePersistence, StateSync};

/// Sensory capability system (Runtime I/O discovery)
pub mod sensory_capabilities;
pub mod sensory_discovery;
pub use sensory_capabilities::{
    AudioInputCapability, AudioOutputCapability, CapabilityError, GestureInputCapability,
    HapticOutputCapability, KeyboardInputCapability, NeuralInputCapability, NeuralOutputCapability,
    PointerInputCapability, SensoryCapabilities, SmellOutputCapability, TasteOutputCapability,
    TouchInputCapability, UIComplexity as SensoryUIComplexity, VisualOutputCapability,
};

/// Instance management (multi-instance support)
pub use instance::{Instance, InstanceError, InstanceId, InstanceRegistry};

/// Session state persistence
pub use session::{
    AccessibilitySettings, SessionError, SessionManager, SessionState, TrustSummary,
};

/// Awakening experience (default touchpoint)
pub use awakening::{AwakeningConfig, AwakeningExperience, AwakeningStage};

/// Awakening coordinator (timeline synchronization)
pub use awakening_coordinator::{
    AwakeningCoordinator, AwakeningTimeline, TimelineEvent, TimelineEventType,
};

/// Compute provider system
pub use compute::{ComputeCapability, ComputeProvider, ComputeRegistry};

/// Toadstool compute integration
pub use toadstool_compute::{CPUFallbackCompute, ToadstoolCompute, ToadstoolServiceInfo};

/// Universal rendering engine
pub use engine::{EngineState, TimeState, UniversalRenderingEngine, ViewMode, Viewport};

/// Event system
pub use event::{EngineEvent, EventBus};

/// Modality system
pub use modality::{
    AccessibilityFeatures, GUIModality, ModalityCapabilities, ModalityRegistry, ModalityTier,
};

/// Frame introspection (content-level self-awareness)
pub use frame_introspection::{
    BindingType, BoundDataObject, ContentAwareness, FrameIntrospection, InteractionCapability,
    InteractionKind, PanelKind, PanelSnapshot,
};

/// Rendering awareness (bidirectional UUI)
pub use rendering_awareness::{
    InteractivityState, MotorCommand, PanelId, RenderingAwareness, RenderingMetrics,
    SelfAssessment, ValidationHealth, VisibilityState,
};

/// SAME DAVE channel model (neuroanatomy: afferent/efferent pathways)
pub use channel::{
    Channel, ChannelDirection, ChannelModality, ChannelRegistry, ChannelSnapshot,
    ClassificationNode, SignalClassifier,
};

/// Sensor system (input abstraction)
pub use sensor::{
    Key, KeyModifiersIpc, Modifiers, MouseButton, Sensor, SensorCapabilities, SensorCapability,
    SensorEvent, SensorEventBatch, SensorEventIpc, SensorRegistry, SensorStats, SensorType,
};

/// Interaction engine (semantic intents, perspectives, inverse pipeline)
pub use interaction::{
    DataObjectId, InputAdapter, InteractionEngine, InteractionEvent, InteractionIntent,
    InteractionResult, InteractionTarget, InversePipeline, OutputModality, Perspective,
    PerspectiveId, PerspectiveSync, SelectionMode, StateChange,
};

/// The petalTongue primal.
pub struct PetalTongue {
    config: PetalTongueConfig,
    state: PrimalState,
}

impl PetalTongue {
    /// Create a new petalTongue instance.
    #[must_use]
    pub const fn new(config: PetalTongueConfig) -> Self {
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
        Ok(
            HealthReport::new(crate::constants::PRIMAL_NAME, env!("CARGO_PKG_VERSION"))
                .with_status(self.health_status()),
        )
    }
}
