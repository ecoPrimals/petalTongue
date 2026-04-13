// SPDX-License-Identifier: AGPL-3.0-or-later
//! Audio System - Substrate-Agnostic Architecture
//!
//! TRUE PRIMAL approach: Discover audio capabilities at runtime,
//! never hardcode external systems (`PipeWire`, `CoreAudio`, WASAPI, etc.)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      AudioManager                            │
//! │              (Discovers ALL backends at runtime)             │
//! └──────────────────────────┬──────────────────────────────────┘
//!                            │
//!           ┌────────────────┼────────────────┬────────────────┐
//!           │                │                │                │
//!           ▼                ▼                ▼                ▼
//!    [Network]        [Software]        [Socket]          [Direct]
//!    compute tier    Pure Rust       Runtime Disc.     Runtime Disc.
//!    Tier 1          Tier 2          Tier 3            Tier 4
//! ```
//!
//! # Philosophy
//!
//! Just like we discover display backends (compute-tier, software, framebuffer),
//! we discover audio backends! NO hardcoding of OS-specific APIs.

pub mod backends;
pub mod compat;
pub mod manager;
pub mod traits;

// Re-exports
pub use compat::AudioSystemV2;
pub use manager::AudioManager;
pub use traits::{AudioBackend, AudioCapabilities, BackendMetadata};
