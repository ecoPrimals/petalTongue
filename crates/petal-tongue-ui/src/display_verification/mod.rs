// SPDX-License-Identifier: AGPL-3.0-only
//! Display Visibility Verification
//!
//! Active verification that the display substrate is actually reaching the user.
//! Part of the bidirectional nervous system.

mod types;
mod verifier;

#[cfg(test)]
mod tests;

pub use types::{DisplayTopology, DisplayVerification, ViewerLocation};
pub use verifier::{continuous_verification, detect_display_topology, verify_display_substrate};
