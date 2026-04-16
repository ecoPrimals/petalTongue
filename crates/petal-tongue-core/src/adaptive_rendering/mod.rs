// SPDX-License-Identifier: AGPL-3.0-or-later
//! Adaptive rendering system for multi-device support
//!
//! This module enables petalTongue to adapt its interface based on device capabilities:
//! - Desktop: Full interface (1400x900, mouse + keyboard)
//! - Phone: Touch-optimized interface (320x568 to 428x926, touch)
//! - Watch: Glance interface (184x224 to 368x448, touch + crown)
//! - CLI: Text-based interface (80x24 terminal)
//!
//! # Philosophy
//!
//! **The interface should adapt to the device, not vice versa.**
//!
//! Don't force users into a single interaction model. Discover what the device
//! can do and render accordingly.

mod detection;
mod renderer_trait;
mod types;

#[cfg(test)]
mod tests;

pub use renderer_trait::AdaptiveRenderer;
pub use types::{
    DeviceType, HapticPrecision, InputMethod, PerformanceTier, RenderingCapabilities,
    RenderingModality, UIComplexity,
};
