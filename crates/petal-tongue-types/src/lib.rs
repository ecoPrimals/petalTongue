// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! Portable visualization data types for petalTongue.
//!
//! This crate contains the core data-binding and threshold types used across
//! the petalTongue ecosystem. It has no platform-specific dependencies, making
//! it suitable for `wasm32-unknown-unknown` and all other Rust targets.
//!
//! Springs push data via these types; the scene engine compiles them into
//! renderable output.

mod data_channel;

pub use data_channel::{DataBinding, ThresholdRange};
