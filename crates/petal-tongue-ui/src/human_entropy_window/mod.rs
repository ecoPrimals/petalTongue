// SPDX-License-Identifier: AGPL-3.0-only
//! Human Entropy Capture UI
//!
//! Provides a user-friendly interface for multi-modal entropy capture.

mod rendering;
mod rendering_helpers;
mod state;
mod types;

#[cfg(test)]
mod tests;

pub use state::HumanEntropyWindow;
