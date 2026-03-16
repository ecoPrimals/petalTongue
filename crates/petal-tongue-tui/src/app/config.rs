// SPDX-License-Identifier: AGPL-3.0-or-later
//! TUI configuration

use petal_tongue_core::constants;
use std::time::Duration;

/// TUI configuration
#[derive(Debug, Clone)]
pub struct TUIConfig {
    /// Tick rate for refresh
    pub tick_rate: Duration,

    /// Enable mouse support
    pub mouse_support: bool,

    /// Start in standalone mode (don't discover primals)
    pub standalone: bool,
}

impl Default for TUIConfig {
    fn default() -> Self {
        Self {
            tick_rate: constants::default_tui_tick_rate(),
            mouse_support: false,
            standalone: false,
        }
    }
}
