// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sensory UI renderer implementations for each complexity level.

mod formatting;
mod immersive_renderer;
mod minimal_renderer;
mod rich_renderer;
mod simple_renderer;
mod standard_renderer;

pub use immersive_renderer::ImmersiveSensoryUI;
pub use minimal_renderer::MinimalSensoryUI;
pub use rich_renderer::RichSensoryUI;
pub use simple_renderer::SimpleSensoryUI;
pub use standard_renderer::StandardSensoryUI;
