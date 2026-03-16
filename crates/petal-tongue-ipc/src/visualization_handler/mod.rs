// SPDX-License-Identifier: AGPL-3.0-or-later
//! Handlers for visualization.render and visualization.render.stream IPC methods.
//!
//! These methods allow springs and other primals to push data for rendering
//! without compile-time coupling -- they discover petalTongue at runtime and
//! send `DataBinding` payloads via JSON-RPC.
//!
//! # Module structure
//!
//! `types` — Request/response DTOs for the IPC contract
//! - `state` — `VisualizationState`: session lifecycle, render, stream, grammar, validate, export, dismiss
//! - `interaction` — `InteractionSubscriberRegistry`: poll-based event subscriptions for springs
//! - `stream` — `binding_id`, `apply_operation`: incremental `DataBinding` updates
//! - `modality` — `compile_modality`: scene graph → SVG/audio/description (internal)

mod interaction;
mod modality;
pub mod pipeline;
mod state;
mod stream;
mod types;

pub use interaction::{
    InteractionEventNotification, InteractionSubscriberRegistry, SensorStreamRegistry,
};
pub use pipeline::PipelineRegistry;
pub use state::{RenderSession, VisualizationState};
pub use stream::{apply_operation, binding_id};
pub use types::{
    BackpressureConfig, ConstraintResult, DashboardRenderRequest, DashboardRenderResponse,
    DismissRequest, DismissResponse, ExportRequest, ExportResponse, GrammarRenderRequest,
    GrammarRenderResponse, InteractionApplyRequest, InteractionApplyResponse, Perspective,
    SessionStatusRequest, SessionStatusResponse, StreamOperation, StreamUpdateRequest,
    StreamUpdateResponse, UiConfig, ValidateRequest, ValidateResponse, VisualizationRenderRequest,
    VisualizationRenderResponse,
};
