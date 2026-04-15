// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Extended headless panel tests: previously-untestable UI logic.
//!
//! These tests exercise panel interactions, motor command processing,
//! data binding visibility, accessibility styling, and multi-frame
//! state evolution through the headless harness.

mod frames_history;
mod introspection_interactions;
mod keyboard_shortcuts;
mod motor_coverage;
mod motor_panels;
mod navigation_input;
mod orchestration_modes;
mod rendering_awareness;
