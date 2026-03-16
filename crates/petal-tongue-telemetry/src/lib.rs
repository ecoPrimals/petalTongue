// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![expect(
    missing_docs,
    reason = "telemetry types documentation tracked for incremental completion"
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! Real-time telemetry and event streaming
//!
//! This crate provides telemetry collection, aggregation, and event streaming
//! for the petalTongue visualization system.
//!
//! # Design Philosophy
//!
//! - **Real-time**: Events streamed as they occur
//! - **Aggregation**: Metrics computed on-the-fly
//! - **Subscriber pattern**: Multiple consumers of telemetry
//! - **Non-blocking**: Async event processing
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐
//! │   Source    │ (BiomeOS, Primals)
//! └──────┬──────┘
//!        │ events
//!        ▼
//! ┌─────────────────┐
//! │ Event Collector │
//! └──────┬──────────┘
//!        │
//!        ├──> Buffer
//!        ├──> Aggregator
//!        └──> Subscribers
//! ```

#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]

mod collector;
mod types;

pub use collector::TelemetryCollector;
pub use types::{PrimalMetrics, TelemetryEvent, TelemetryMetrics, TelemetrySubscriber};
