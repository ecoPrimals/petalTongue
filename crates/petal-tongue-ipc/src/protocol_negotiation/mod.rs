// SPDX-License-Identifier: AGPL-3.0-or-later
//! G65 Protocol Negotiation for petalTongue.
//!
//! Single-socket protocol selection at connection time. Replaces the C2
//! dual-socket pattern with a negotiation header on one socket.
//!
//! ## Wire Protocol
//!
//! ```text
//! Client → Server: "PROTOCOLS: tarpc,jsonrpc\n"
//! Server → Client: "PROTOCOL: tarpc\n"
//! [Connection proceeds with selected protocol framing]
//! ```
//!
//! ## Backward Compatibility
//!
//! If the first bytes are NOT a `PROTOCOLS:` header, the server assumes
//! JSON-RPC (default protocol). This means legacy clients continue to work
//! without modification.

mod negotiate;
mod server;
mod wire;

#[cfg(test)]
mod tests;

pub use negotiate::{NegotiationResult, negotiate_client, negotiate_server, select_protocol};
pub use server::{NegotiateServer, NegotiateServerError};
pub use wire::{NegotiationError, ProtocolId, ProtocolRequest, ProtocolResponse};
