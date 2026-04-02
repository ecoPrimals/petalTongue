// SPDX-License-Identifier: AGPL-3.0-or-later
//! Concrete input adapter and inverse pipeline implementations.
//!
//! These bridge the existing [`Sensor`](petal_tongue_core::Sensor) system to
//! the [`InputAdapter`](petal_tongue_core::interaction::InputAdapter) trait from the interaction engine.

pub mod agent_adapter;
pub mod audio_inverse;
pub mod keyboard_adapter;
pub mod pointer_adapter;
pub mod switch_adapter;
pub mod visual_inverse;

pub use agent_adapter::AgentInputAdapter;
pub use audio_inverse::AudioInversePipeline;
pub use keyboard_adapter::KeyboardAdapter;
pub use pointer_adapter::PointerAdapter;
pub use switch_adapter::{ScanMode, SwitchInputAdapter};
pub use visual_inverse::VisualInversePipeline;
