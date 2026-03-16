// SPDX-License-Identifier: AGPL-3.0-or-later
//! Visualization session state and handler logic.
//!
//! Manages active visualization sessions from springs/primals, including
//! render, stream updates, grammar compilation, validation, export, and dismiss.

mod queries;
mod render_handlers;
mod session_lifecycle;
mod stream_handler;
mod types;

#[cfg(test)]
mod tests;

pub use types::{RenderSession, VisualizationState};
