// SPDX-License-Identifier: AGPL-3.0-or-later
//! Panel Registry - Dynamic panel type registration and instantiation
//!
//! This module provides the infrastructure for registering custom panel types
//! and creating panel instances from scenario configuration.
//!
//! # Evolution Note
//! This system emerged from implementing Doom (Gap #1 in `DOOM_GAP_LOG.md`).
//! We needed a way to map `"doom_game"` in JSON to `DoomPanel` creation.

mod factory;
mod registry;
mod types;

#[cfg(test)]
mod tests;

pub use factory::{PanelFactory, PanelFactoryImpl, PanelInstanceImpl};
pub use registry::PanelRegistry;
pub use types::{PanelAction, PanelError, PanelInstance, Result};
