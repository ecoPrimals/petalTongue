// SPDX-License-Identifier: AGPL-3.0-only
//! State synchronization across devices
//!
//! This module enables petalTongue to maintain consistent state across
//! multiple devices (desktop, phone, watch).

mod persistence;
mod sync;
mod types;

#[cfg(test)]
mod tests;

pub use persistence::{LocalStatePersistence, StatePersistence};
pub use sync::StateSync;
pub use types::DeviceState;
