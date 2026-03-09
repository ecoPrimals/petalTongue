// SPDX-License-Identifier: AGPL-3.0-only
//! Concrete input adapter implementations.
//!
//! These bridge the existing [`Sensor`](petal_tongue_core::Sensor) system to
//! the [`InputAdapter`](petal_tongue_core::interaction::InputAdapter) trait from the interaction engine.

pub mod keyboard_adapter;
pub mod pointer_adapter;
pub mod visual_inverse;

pub use keyboard_adapter::KeyboardAdapter;
pub use pointer_adapter::PointerAdapter;
pub use visual_inverse::VisualInversePipeline;
