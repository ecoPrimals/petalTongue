// SPDX-License-Identifier: AGPL-3.0-only
//! Sensory-Based Adaptive UI System
//!
//! This module replaces device-type based rendering with capability-based
//! rendering. Instead of asking "what device is this?", we ask "what
//! capabilities does this device have?"
//!
//! # Architecture
//!
//! - **Discover** capabilities at runtime (visual, audio, haptic, inputs)
//! - **Determine** UI complexity from capabilities (Minimal → Immersive)
//! - **Adapt** rendering based on complexity
//! - **Hot-reload** when capabilities change (VR headset plugged in)

mod manager;
mod renderers;
#[cfg(test)]
mod tests;

pub use manager::{SensoryUIManager, SensoryUIRenderer};
