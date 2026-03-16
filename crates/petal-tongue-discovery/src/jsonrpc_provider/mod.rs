// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC 2.0 Provider - PRIMARY PRIMAL PROTOCOL
//!
//! Line-delimited JSON-RPC 2.0 over Unix sockets for fast, secure, port-free
//! inter-primal communication.

mod provider;
mod types;

#[cfg(test)]
mod tests;

pub use types::JsonRpcProvider;
