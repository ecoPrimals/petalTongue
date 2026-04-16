// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::manual_async_fn)] // matches `DiscoveryBackend` desugared async signatures
//! biomeOS Discovery Backend
//!
//! Implements capability-based discovery via biomeOS Neural API.
//! This is the primary discovery mechanism in production.

mod backend;
mod client;
mod types;

#[cfg(test)]
mod tests;

pub use backend::BiomeOsBackend;
pub use types::BiomeOSDiscoveryEvent;
