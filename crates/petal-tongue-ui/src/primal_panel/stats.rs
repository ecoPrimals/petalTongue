// SPDX-License-Identifier: AGPL-3.0-or-later
//! Statistics computation for primal panel (counts by health status).

use crate::biomeos_integration::{Health, Primal};

/// Computes primal counts by health status.
/// Returns (total, healthy, degraded, error).
#[must_use]
pub fn compute_primal_stats(primals: &[Primal]) -> (usize, usize, usize, usize) {
    let total = primals.len();
    let healthy = primals
        .iter()
        .filter(|p| p.health == Health::Healthy)
        .count();
    let degraded = primals
        .iter()
        .filter(|p| p.health == Health::Degraded)
        .count();
    let error = primals
        .iter()
        .filter(|p| p.health == Health::Offline)
        .count();
    (total, healthy, degraded, error)
}
