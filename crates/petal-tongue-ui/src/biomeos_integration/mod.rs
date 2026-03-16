// SPDX-License-Identifier: AGPL-3.0-or-later
//! biomeOS Integration - Visualization Data Provider
//!
//! Provides capability-based discovery and integration with biomeOS for device
//! and niche management UI.
//!
//! # TRUE PRIMAL Principles
//!
//! - **Zero Hardcoding**: Discovers biomeOS by capability, not by name
//! - **Graceful Degradation**: Falls back to mock data when biomeOS unavailable
//! - **Self-Knowledge**: Announces own capabilities to ecosystem
//! - **Runtime Discovery**: No compile-time dependencies on biomeOS
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ petalTongue                                                 │
//! │  ├─ DevicePanel ────────────┐                              │
//! │  ├─ PrimalPanel ────────────┼─→ BiomeOSProvider           │
//! │  └─ NicheDesigner ──────────┘      │                       │
//! │                                     ↓                       │
//! │                              [Event Stream]                 │
//! │                                     ↓                       │
//! │                              UIEventHandler                 │
//! └─────────────────────────────────────────────────────────────┘
//!                                      ↓
//!                          (Unix Socket / WebSocket)
//!                                      ↓
//! ┌─────────────────────────────────────────────────────────────┐
//! │ biomeOS (Capability: "device.management")                   │
//! │  ├─ Device Discovery                                        │
//! │  ├─ Primal Registry                                         │
//! │  ├─ Niche Orchestration                                     │
//! │  └─ AI Suggestions                                          │
//! └─────────────────────────────────────────────────────────────┘
//! ```

mod events;
mod provider;
mod provider_trait;
pub mod sse;
mod types;

#[cfg(test)]
mod tests;

// Re-export public API (unchanged from original single-file module)
pub use events::BiomeOSEvent;
pub use provider::BiomeOSProvider;
pub use sse::{EcosystemEvent, SseConnectionState, SseEventConsumer};
pub use types::{Device, DeviceStatus, DeviceType, Health, NicheTemplate, Primal};
