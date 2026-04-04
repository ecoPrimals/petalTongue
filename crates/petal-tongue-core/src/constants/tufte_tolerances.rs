// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tufte visualization tolerances (absorbed from ludoSpring V14).
//!
//! These constants define maximum acceptable deviations for golden pixel
//! testing and data-ink ratio validation.

/// Maximum fraction of non-data-ink pixels tolerated in a visualization.
/// A ratio of 0.01 means ≤1% chartjunk is acceptable.
pub const UI_DATA_INK_TOL: f64 = 0.01;

/// Maximum fraction of viewport area that may remain uncovered by data.
/// A ratio of 0.05 means ≥95% coverage is required.
pub const UI_COVERAGE_TOL: f64 = 0.05;

/// Maximum tolerable distance error in raycaster depth calculations.
pub const RAYCASTER_DISTANCE_TOL: f64 = 0.001;

/// Minimum coherence threshold for noise-based procedural generation.
pub const NOISE_COHERENCE_TOL: f64 = 0.01;
