// SPDX-License-Identifier: AGPL-3.0-or-later
//! UI Panel Rendering for petalTongue
//!
//! Extracted from app.rs to reduce complexity and improve maintainability.
//! Contains all panel rendering logic:
//! - Top menu bar
//! - Controls panel (left)
//! - Audio info panel (right)
//! - Primal details panel (right)
//!
//! Each panel is a pure function that takes app state and renders to egui UI.

mod builders;
mod layout;
mod primal_details;

pub use builders::{
    render_audio_panel, render_capability_panel, render_controls_panel,
    render_primal_details_panel, render_top_menu_bar,
};
