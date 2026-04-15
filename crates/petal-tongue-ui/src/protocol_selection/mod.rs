// SPDX-License-Identifier: AGPL-3.0-or-later
//! Protocol selection logic for inter-primal communication
//!
//! Implements ecosystem-standard protocol priority:
//! 1. tarpc (PRIMARY) - High-performance binary RPC
//! 2. JSON-RPC (SECONDARY) - Universal, debuggable
//! 3. HTTPS (FALLBACK) - External/browser access

mod connect;
mod connection;
mod https_client;
mod parse;
mod protocol;

pub use connect::connect_with_priority;
pub use connection::PrimalConnection;
pub use https_client::HttpsClient;
pub use parse::{parse_capabilities_from_json, parse_health_from_json};
pub use protocol::{DetectedProtocol, Protocol, detect_protocol, https_fallback_urls};

#[cfg(test)]
mod tests;
