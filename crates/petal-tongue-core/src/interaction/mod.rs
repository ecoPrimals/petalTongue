// SPDX-License-Identifier: AGPL-3.0-or-later
//! # Interaction Engine
//!
//! Bidirectional, modality-agnostic interaction system for petalTongue.
//!
//! The interaction engine translates device events (pointer activation, key presses,
//! voice commands, Braille routing keys) into semantic intents (Select, Inspect,
//! Navigate) that are independent of input modality. The generalized inverse
//! pipeline resolves these intents to data-space targets that are
//! perspective-invariant.
//!
//! ## The "6 vs 9" Solution
//!
//! Two users with different perspectives (sighted + display, blind + audio) both
//! resolve to the same [`DataObjectId`] when interacting with the same data.
//! Selection and focus operate on data identity, not on rendered primitives.
//!
//! ## Architecture
//!
//! ```text
//! SensorEvent (device)
//!     -> InputAdapter.translate() -> InteractionIntent (semantic)
//!     -> InversePipeline.resolve() -> InteractionTarget (data-space)
//!     -> InteractionEngine.apply() -> StateChange
//!     -> Broadcast to perspectives + IPC
//! ```
//!
//! See `specs/INTERACTION_ENGINE_ARCHITECTURE.md` for the full specification.

pub mod adapter;
pub mod adapters;
pub mod engine;
pub mod input_adapter_impl;
pub mod intent;
pub mod inverse;
pub mod inverse_pipeline_impl;
pub mod perspective;
pub mod result;
pub mod target;

#[cfg(test)]
mod mock_adapters;

// Re-export core types at module level for ergonomic access.
pub use adapter::{InputAdapter, InputModality, InteractionCapability, InteractionContext};
pub use adapters::{
    AgentInputAdapter, AudioInversePipeline, KeyboardAdapter, PointerAdapter, ScanMode,
    SwitchInputAdapter, VisualInversePipeline,
};
pub use engine::InteractionEngine;
pub use input_adapter_impl::InputAdapterImpl;
pub use intent::{
    AnnotationContent, InspectionDepth, InteractionIntent, ManipulationOp, NavigationDirection,
    SelectionMode,
};
pub use inverse::{InversePipeline, NoOpInversePipeline};
pub use inverse_pipeline_impl::InversePipelineImpl;
pub use perspective::{
    Axis, Orientation, OutputModality, Perspective, PerspectiveId, PerspectiveSync,
    PerspectiveViewport,
};
pub use result::{
    DataMutation, InteractionClock, InteractionEvent, InteractionResult, StateChange,
};
pub use target::{
    BoundingBox, DataObjectId, DataRow, DataSourceId, FilterExpr, GrammarId, InteractionTarget,
    PrimitiveId,
};
