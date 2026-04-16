// SPDX-License-Identifier: AGPL-3.0-or-later
//! [`AdaptiveRenderer`] trait.

use super::types::RenderingCapabilities;

/// Trait for adaptive rendering
pub trait AdaptiveRenderer {
    /// Check if this renderer can handle the given capabilities
    fn supports(&self, caps: &RenderingCapabilities) -> bool;

    /// Get renderer priority (higher = preferred)
    fn priority(&self) -> i32 {
        0
    }
}
