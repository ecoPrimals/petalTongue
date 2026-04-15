// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sensory Capability Matrix — the structured answer to "what can this user do?"
//!
//! When ludoSpring asks "can this user play my game?" or Squirrel asks "how do
//! I navigate for a blind user?", this module provides the negotiated answer.
//!
//! The matrix crosses **input capabilities** (what the user/agent can provide)
//! with **output capabilities** (what petalTongue can render). Each validated
//! cell represents a tested interaction path.

mod capability_sets;
mod matrix;

#[cfg(test)]
mod tests;

pub use capability_sets::{InputCapabilitySet, OutputCapabilitySet};
pub use matrix::{InteractionPattern, SensoryCapabilityMatrix, ValidatedPath};
