// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![expect(missing_docs, reason = "incremental documentation in progress")]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! Doom Core - Doom integration for petalTongue
//!
//! This crate provides the infrastructure for running Doom within petalTongue.
//! It demonstrates petalTongue's platform capabilities through test-driven evolution.
//!
//! # Architecture
//!
//! The Doom integration is designed to expose gaps in petalTongue's architecture:
//! - Panel lifecycle management
//! - Input focus and routing
//! - Performance budgets
//! - Resource coordination
//! - Asset loading
//! - Audio mixing
//!
//! As we implement Doom, we discover and solve these gaps, evolving petalTongue
//! into a robust platform for ANY embedded application.
//!
//! # Phase 1.1: WAD Parsing & Map Display
//!
//! We start by loading a real Doom WAD file and displaying the map geometry.
//! This validates our asset loading and rendering capabilities.
//!
//! ## License
//!
//! Code: AGPL-3.0-or-later (see SPDX header)
//! Game mechanics: ORC (Open RPG Creative License)

pub mod error;
pub mod instance;
pub mod key;
pub mod map_renderer;
pub mod raycast_renderer;
pub mod state;
pub mod wad_loader;

#[cfg(test)]
mod tests;

// Re-exports for public API stability
pub use error::{DoomError, Result};
pub use instance::DoomInstance;
pub use key::DoomKey;
pub use state::{DoomState, GameStats, ViewMode};
