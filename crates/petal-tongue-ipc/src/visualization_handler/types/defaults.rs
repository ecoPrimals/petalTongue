// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared `serde` default helpers for visualization IPC DTOs.

pub fn default_modality() -> String {
    "svg".into()
}

pub const fn default_true() -> bool {
    true
}

pub const fn default_dashboard_columns() -> usize {
    3
}
