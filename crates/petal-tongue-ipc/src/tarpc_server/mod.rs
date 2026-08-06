// SPDX-License-Identifier: AGPL-3.0-or-later
//! tarpc UDS server for petalTongue (C2 dual-socket pattern).
//!
//! Binds `petaltongue.tarpc.sock` alongside the JSON-RPC `.sock`,
//! serving the `PetalTongueRpc` service trait over binary bincode framing.

mod serve;

#[cfg(test)]
mod tests;

pub use serve::{TarpcServer, TarpcServerError};
