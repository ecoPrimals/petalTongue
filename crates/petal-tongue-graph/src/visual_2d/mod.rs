// SPDX-License-Identifier: AGPL-3.0-only
//! 2D Visual Renderer
//!
//! Renders graph topology as 2D graphics using egui.
//! Supports animation of flow particles and node pulses.

mod animation;
mod drawing;
mod interaction;
mod nodes;
mod renderer;
mod stats;
mod types;

#[cfg(test)]
mod tests;

pub use renderer::Visual2DRenderer;
pub(crate) use types::EdgeDraft;
