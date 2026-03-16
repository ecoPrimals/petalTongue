// SPDX-License-Identifier: AGPL-3.0-or-later
//! # tarpc Client for petalTongue
//!
//! High-performance primal-to-primal RPC client.

mod client;
mod types;

#[cfg(test)]
mod tests;

pub use types::{TarpcClient, TarpcClientError, TarpcResult};
