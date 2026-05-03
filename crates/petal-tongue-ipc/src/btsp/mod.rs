// SPDX-License-Identifier: AGPL-3.0-or-later
//! BearDog-derived Transport Security Profile (BTSP).
//!
//! Phase 1: insecure startup guard, family-scoped socket names, and visualization symlinks.
//! Phase 2: handshake enforcement via security provider delegation.
//! Phase 3: encrypted frame I/O using ChaCha20-Poly1305 AEAD after `btsp.negotiate`.

mod client;
/// Typed errors for BTSP handshake operations.
pub mod error;
pub(crate) mod framing;
mod json_line;
/// Phase 3: encrypted frame I/O and session key derivation.
pub mod phase3;
mod server;
mod types;

#[cfg(test)]
mod tests;

pub use types::{
    BtspGuardError, BtspHandshakeConfig, BtspPosture, HandshakePolicy, HandshakeResult,
    current_btsp_posture, domain_symlink_filename, handshake_policy, log_handshake_policy,
    socket_filename, validate_insecure_guard,
};

pub use server::{perform_server_handshake, perform_server_handshake_split};

pub use json_line::{relay_json_line_handshake, relay_json_line_handshake_split};
