// SPDX-License-Identifier: AGPL-3.0-or-later
//! Trust dashboard display state types (headless-testable, no egui dependency for logic).

use std::collections::HashMap;

/// Pre-computed row for a single trust level in the distribution table.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustLevelRow {
    pub label: String,
    pub count: usize,
    pub percentage: f32,
    pub emoji: &'static str,
    pub color: [u8; 4],
}

/// Pre-computed display data for the average trust indicator.
#[derive(Debug, Clone, PartialEq)]
pub struct AverageTrustDisplay {
    pub value: f64,
    pub emoji: &'static str,
    pub color: [u8; 4],
    pub label: &'static str,
    pub sound_name: &'static str,
}

/// Complete pre-computed display state for the trust dashboard.
#[derive(Debug, Clone)]
pub struct TrustDisplayState {
    pub rows: Vec<TrustLevelRow>,
    pub total_primals: usize,
    pub average: Option<AverageTrustDisplay>,
    pub family_count: usize,
    pub unique_families: usize,
    pub last_update_label: String,
}

/// Intent produced by render interactions. The caller (event loop) decides
/// what to do with these rather than the render method having side effects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrustIntent {
    PlayAudio { sound: String },
}

/// Summary of trust information across the network
#[derive(Default, Clone)]
pub struct TrustSummary {
    pub trust_distribution: HashMap<String, usize>,
    pub total_primals: usize,
    pub family_count: usize,
    pub unique_families: usize,
    pub average_trust: Option<f64>,
}
