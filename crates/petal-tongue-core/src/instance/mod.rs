// SPDX-License-Identifier: AGPL-3.0-or-later
//! Instance management for petalTongue
//!
//! This module provides the foundation for managing multiple petalTongue instances,
//! enabling instance tracking, discovery, and coordination.

mod entity;
mod lifecycle;
mod registry;
mod types;

#[cfg(test)]
mod tests;

pub use entity::Instance;
pub use registry::InstanceRegistry;
pub use types::{InstanceError, InstanceId};
