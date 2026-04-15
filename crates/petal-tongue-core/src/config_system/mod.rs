// SPDX-License-Identifier: AGPL-3.0-or-later
//! Platform-Agnostic Configuration System
//!
//! TRUE PRIMAL principle: Zero hardcoding, XDG-compliant, environment-driven
//!
//! This module provides comprehensive configuration management that adapts
//! to the host environment without hardcoded assumptions.

mod loader;
mod types;

#[cfg(test)]
mod tests;

pub use types::{
    Config, ConfigError, DiscoveryConfig, NetworkConfig, PathsConfig, PerformanceConfig,
    ThresholdsConfig,
};
