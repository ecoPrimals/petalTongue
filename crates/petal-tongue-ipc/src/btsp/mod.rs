// SPDX-License-Identifier: AGPL-3.0-or-later
//! BearDog Transport Security Profile (BTSP).
//!
//! Phase 1: insecure startup guard, family-scoped socket names, and visualization symlinks.
//! Phase 2: handshake enforcement via BearDog session delegation.

mod client;
mod framing;
mod server;
mod types;

#[cfg(test)]
mod tests;

pub use types::{
    BtspGuardError, BtspHandshakeConfig, BtspPosture, HandshakePolicy, current_btsp_posture,
    domain_symlink_filename, handshake_policy, log_handshake_policy, socket_filename,
    validate_insecure_guard,
};

pub use server::{perform_server_handshake, perform_server_handshake_split};
