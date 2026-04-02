// SPDX-License-Identifier: AGPL-3.0-or-later
//! Neural API-based visualization data provider
//!
//! Connects to biomeOS Neural API for unified primal discovery and proprioception.
//! This is the PREFERRED provider as Neural API is the central coordinator.

mod parse;
mod provider;

#[cfg(test)]
mod mock_server;

#[cfg(test)]
mod tests;

pub use provider::NeuralApiProvider;
