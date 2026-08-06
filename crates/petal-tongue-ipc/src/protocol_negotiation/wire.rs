// SPDX-License-Identifier: AGPL-3.0-or-later
//! Wire format types for G65 protocol negotiation.

use std::fmt;

/// Protocol identifiers supported by petalTongue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProtocolId {
    /// JSON-RPC 2.0 (text-based, universal, backward-compatible default).
    #[default]
    JsonRpc,
    /// tarpc binary bincode framing (sub-ms, Rust-to-Rust).
    Tarpc,
}

impl ProtocolId {
    /// All protocols this build supports, in server preference order.
    #[must_use]
    pub fn supported() -> Vec<Self> {
        vec![Self::Tarpc, Self::JsonRpc]
    }

    /// Wire name used in the negotiation header.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::JsonRpc => "jsonrpc",
            Self::Tarpc => "tarpc",
        }
    }

    /// Parse from wire name (case-insensitive).
    #[must_use]
    pub fn from_wire_name(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "jsonrpc" | "json-rpc" | "json_rpc" => Some(Self::JsonRpc),
            "tarpc" => Some(Self::Tarpc),
            _ => None,
        }
    }
}

impl fmt::Display for ProtocolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.wire_name())
    }
}

/// Client → Server request listing supported protocols.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolRequest {
    /// Protocols the client supports, ordered by client preference.
    pub supported: Vec<ProtocolId>,
}

impl ProtocolRequest {
    /// Serialize to wire format: `"PROTOCOLS: tarpc,jsonrpc\n"`.
    #[must_use]
    pub fn to_wire(&self) -> String {
        let names: Vec<&str> = self.supported.iter().map(|p| p.wire_name()).collect();
        format!("PROTOCOLS: {}\n", names.join(","))
    }

    /// Parse from wire format.
    pub fn from_wire(line: &str) -> Result<Self, NegotiationError> {
        let trimmed = line.trim();
        let payload = trimmed
            .strip_prefix("PROTOCOLS: ")
            .ok_or(NegotiationError::InvalidPrefix)?;

        let supported: Vec<ProtocolId> = payload
            .split(',')
            .filter_map(|s| ProtocolId::from_wire_name(s.trim()))
            .collect();

        if supported.is_empty() {
            return Err(NegotiationError::NoValidProtocols);
        }

        Ok(Self { supported })
    }
}

/// Server → Client response selecting a protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolResponse {
    /// The protocol the server selected.
    pub selected: ProtocolId,
}

impl ProtocolResponse {
    /// Serialize to wire format: `"PROTOCOL: tarpc\n"`.
    #[must_use]
    pub fn to_wire(&self) -> String {
        format!("PROTOCOL: {}\n", self.selected.wire_name())
    }

    /// Parse from wire format.
    pub fn from_wire(line: &str) -> Result<Self, NegotiationError> {
        let trimmed = line.trim();
        let name = trimmed
            .strip_prefix("PROTOCOL: ")
            .ok_or(NegotiationError::InvalidPrefix)?;

        let selected =
            ProtocolId::from_wire_name(name).ok_or(NegotiationError::UnknownProtocol)?;

        Ok(Self { selected })
    }
}

/// Errors during protocol negotiation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NegotiationError {
    /// Line does not start with the expected prefix.
    #[error("invalid negotiation prefix")]
    InvalidPrefix,
    /// No recognized protocols in the request.
    #[error("no valid protocols in request")]
    NoValidProtocols,
    /// Protocol name not recognized.
    #[error("unknown protocol in response")]
    UnknownProtocol,
    /// I/O error during negotiation.
    #[error("negotiation I/O: {0}")]
    Io(String),
    /// Timeout waiting for negotiation.
    #[error("negotiation timed out")]
    Timeout,
}
