// SPDX-License-Identifier: AGPL-3.0-only
//! Machine-readable status reporter for AI and external system inspection
//!
//! This module makes petalTongue's internal state observable to:
//! - AI systems that need to understand what's happening
//! - External monitoring tools
//! - Automated testing systems
//! - Other primals that need to adjust their behavior

mod reporter;
mod types;

pub use reporter::StatusReporter;
pub use types::{
    AudioEvent, AudioProviderInfo, AudioStatus, DiscoveryStatus, Issue, ModalityState,
    ModalityStatus, StatusEvent, SystemStatus, UIStatus,
};
