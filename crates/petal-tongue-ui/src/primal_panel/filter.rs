// SPDX-License-Identifier: AGPL-3.0-or-later
//! Primal filter types for the primal panel UI.

/// Primal filter options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimalFilter {
    /// Show all primals
    All,
    /// Show only healthy primals
    Healthy,
    /// Show only degraded primals
    Degraded,
}
