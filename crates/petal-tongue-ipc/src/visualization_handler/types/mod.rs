// SPDX-License-Identifier: AGPL-3.0-or-later
//! Request and response types for visualization IPC methods.
//!
//! These DTOs define the JSON-RPC contract for visualization.render,
//! visualization.render.stream, visualization.render.grammar, visualization.validate,
//! visualization.export, visualization.dismiss, and visualization.interact.*.

mod dashboard;
mod defaults;
mod dismiss;
mod export;
mod grammar;
mod interact;
mod render;
mod session;
mod stream;
mod validate;

pub use dashboard::{DashboardRenderRequest, DashboardRenderResponse};
pub use dismiss::{DismissRequest, DismissResponse};
pub use export::{ExportRequest, ExportResponse};
pub use grammar::{GrammarRenderRequest, GrammarRenderResponse};
pub use interact::{InteractionApplyRequest, InteractionApplyResponse, Perspective};
pub use render::{UiConfig, VisualizationRenderRequest, VisualizationRenderResponse};
pub use session::{SessionStatusRequest, SessionStatusResponse};
pub use stream::{BackpressureConfig, StreamOperation, StreamUpdateRequest, StreamUpdateResponse};
pub use validate::{ConstraintResult, ValidateRequest, ValidateResponse};
