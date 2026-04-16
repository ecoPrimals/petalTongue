// SPDX-License-Identifier: AGPL-3.0-or-later
//! Panel factory trait, enum dispatch for built-in panels, and [`PanelFactory`] impls.

use crate::scenario::CustomPanelConfig;

use super::types::{PanelAction, PanelInstance, Result};

#[cfg(test)]
use crate::panel_registry::tests::panel_test_support;

/// Factory for creating panel instances
///
/// Each custom panel type implements this trait to enable
/// registration and instantiation from scenarios.
pub trait PanelFactory: Send + Sync {
    /// Get the panel type identifier (e.g., "`doom_game`")
    fn panel_type(&self) -> &str;

    /// Create a new panel instance from configuration
    ///
    /// # Errors
    ///
    /// Returns an error if panel creation fails (missing resources, invalid config, initialization error).
    fn create(&self, config: &CustomPanelConfig) -> Result<PanelInstanceImpl>;

    /// Get human-readable description
    fn description(&self) -> &'static str {
        "Custom panel"
    }
}

/// Enum dispatch for all built-in [`PanelInstance`] implementations.
pub enum PanelInstanceImpl {
    /// Doom game panel (feature `doom`).
    #[cfg(feature = "doom")]
    Doom(crate::panels::doom_factory::DoomPanelWrapper),
    /// Doom statistics overlay (feature `doom`).
    #[cfg(feature = "doom")]
    DoomStats(crate::panels::doom_stats_panel::DoomStatsPanel),
    /// Neural API / system metrics panel.
    Metrics(crate::panels::metrics_panel::MetricsPanel),
    /// NUCLEUS proprioception panel.
    Proprioception(crate::panels::proprioception_panel::ProprioceptionPanel),
    /// Test-only mock panel.
    #[cfg(test)]
    TestMock(panel_test_support::MockPanel),
}

impl PanelInstance for PanelInstanceImpl {
    fn render(&mut self, ui: &mut egui::Ui) {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::render(p, ui),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::render(p, ui),
            Self::Metrics(p) => PanelInstance::render(p, ui),
            Self::Proprioception(p) => PanelInstance::render(p, ui),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::render(p, ui),
        }
    }

    fn title(&self) -> &str {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::title(p),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::title(p),
            Self::Metrics(p) => PanelInstance::title(p),
            Self::Proprioception(p) => PanelInstance::title(p),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::title(p),
        }
    }

    fn update(&mut self) {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::update(p),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::update(p),
            Self::Metrics(p) => PanelInstance::update(p),
            Self::Proprioception(p) => PanelInstance::update(p),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::update(p),
        }
    }

    fn on_event(&mut self, event: &egui::Event) {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::on_event(p, event),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::on_event(p, event),
            Self::Metrics(p) => PanelInstance::on_event(p, event),
            Self::Proprioception(p) => PanelInstance::on_event(p, event),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::on_event(p, event),
        }
    }

    fn wants_keyboard_input(&self) -> bool {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::wants_keyboard_input(p),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::wants_keyboard_input(p),
            Self::Metrics(p) => PanelInstance::wants_keyboard_input(p),
            Self::Proprioception(p) => PanelInstance::wants_keyboard_input(p),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::wants_keyboard_input(p),
        }
    }

    fn wants_mouse_input(&self) -> bool {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::wants_mouse_input(p),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::wants_mouse_input(p),
            Self::Metrics(p) => PanelInstance::wants_mouse_input(p),
            Self::Proprioception(p) => PanelInstance::wants_mouse_input(p),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::wants_mouse_input(p),
        }
    }

    fn wants_exclusive_input(&self) -> bool {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::wants_exclusive_input(p),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::wants_exclusive_input(p),
            Self::Metrics(p) => PanelInstance::wants_exclusive_input(p),
            Self::Proprioception(p) => PanelInstance::wants_exclusive_input(p),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::wants_exclusive_input(p),
        }
    }

    fn input_priority(&self) -> u8 {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::input_priority(p),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::input_priority(p),
            Self::Metrics(p) => PanelInstance::input_priority(p),
            Self::Proprioception(p) => PanelInstance::input_priority(p),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::input_priority(p),
        }
    }

    fn on_keyboard_event(&mut self, ctx: &egui::Context) -> crate::focus_manager::InputAction {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::on_keyboard_event(p, ctx),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::on_keyboard_event(p, ctx),
            Self::Metrics(p) => PanelInstance::on_keyboard_event(p, ctx),
            Self::Proprioception(p) => PanelInstance::on_keyboard_event(p, ctx),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::on_keyboard_event(p, ctx),
        }
    }

    fn on_mouse_event(&mut self, ctx: &egui::Context) -> crate::focus_manager::InputAction {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::on_mouse_event(p, ctx),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::on_mouse_event(p, ctx),
            Self::Metrics(p) => PanelInstance::on_mouse_event(p, ctx),
            Self::Proprioception(p) => PanelInstance::on_mouse_event(p, ctx),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::on_mouse_event(p, ctx),
        }
    }

    fn on_open(&mut self) -> crate::error::Result<()> {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::on_open(p),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::on_open(p),
            Self::Metrics(p) => PanelInstance::on_open(p),
            Self::Proprioception(p) => PanelInstance::on_open(p),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::on_open(p),
        }
    }

    fn on_close(&mut self) -> crate::error::Result<()> {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::on_close(p),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::on_close(p),
            Self::Metrics(p) => PanelInstance::on_close(p),
            Self::Proprioception(p) => PanelInstance::on_close(p),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::on_close(p),
        }
    }

    fn on_pause(&mut self) {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::on_pause(p),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::on_pause(p),
            Self::Metrics(p) => PanelInstance::on_pause(p),
            Self::Proprioception(p) => PanelInstance::on_pause(p),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::on_pause(p),
        }
    }

    fn on_resume(&mut self) {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::on_resume(p),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::on_resume(p),
            Self::Metrics(p) => PanelInstance::on_resume(p),
            Self::Proprioception(p) => PanelInstance::on_resume(p),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::on_resume(p),
        }
    }

    fn on_error(&mut self, error: &dyn std::error::Error) -> PanelAction {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::on_error(p, error),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::on_error(p, error),
            Self::Metrics(p) => PanelInstance::on_error(p, error),
            Self::Proprioception(p) => PanelInstance::on_error(p, error),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::on_error(p, error),
        }
    }

    fn can_save_state(&self) -> bool {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::can_save_state(p),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::can_save_state(p),
            Self::Metrics(p) => PanelInstance::can_save_state(p),
            Self::Proprioception(p) => PanelInstance::can_save_state(p),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::can_save_state(p),
        }
    }

    fn can_restore_state(&self) -> bool {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::can_restore_state(p),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::can_restore_state(p),
            Self::Metrics(p) => PanelInstance::can_restore_state(p),
            Self::Proprioception(p) => PanelInstance::can_restore_state(p),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::can_restore_state(p),
        }
    }

    fn save_state(&self) -> crate::error::Result<serde_json::Value> {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::save_state(p),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::save_state(p),
            Self::Metrics(p) => PanelInstance::save_state(p),
            Self::Proprioception(p) => PanelInstance::save_state(p),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::save_state(p),
        }
    }

    fn restore_state(&mut self, state: serde_json::Value) -> crate::error::Result<()> {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::restore_state(p, state),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::restore_state(p, state),
            Self::Metrics(p) => PanelInstance::restore_state(p, state),
            Self::Proprioception(p) => PanelInstance::restore_state(p, state),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::restore_state(p, state),
        }
    }

    fn is_closable(&self) -> bool {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::is_closable(p),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::is_closable(p),
            Self::Metrics(p) => PanelInstance::is_closable(p),
            Self::Proprioception(p) => PanelInstance::is_closable(p),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::is_closable(p),
        }
    }

    fn is_pausable(&self) -> bool {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => PanelInstance::is_pausable(p),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => PanelInstance::is_pausable(p),
            Self::Metrics(p) => PanelInstance::is_pausable(p),
            Self::Proprioception(p) => PanelInstance::is_pausable(p),
            #[cfg(test)]
            Self::TestMock(p) => PanelInstance::is_pausable(p),
        }
    }
}

/// Enum dispatch for all built-in [`PanelFactory`] implementations.
pub enum PanelFactoryImpl {
    /// Doom game panel factory (feature `doom`).
    #[cfg(feature = "doom")]
    Doom(crate::panels::doom_factory::DoomPanelFactory),
    /// Doom stats panel factory (feature `doom`).
    #[cfg(feature = "doom")]
    DoomStats(crate::panels::doom_stats_panel::DoomStatsPanelFactory),
    /// Metrics panel factory.
    Metrics(crate::panels::metrics_panel::MetricsPanelFactory),
    /// Proprioception panel factory.
    Proprioception(crate::panels::proprioception_panel::ProprioceptionPanelFactory),
    /// Test-only mock factory.
    #[cfg(test)]
    TestMock(panel_test_support::MockPanelFactory),
    /// Test-only factory that always fails creation.
    #[cfg(test)]
    TestFailing(panel_test_support::FailingPanelFactory),
}

impl PanelFactory for PanelFactoryImpl {
    fn panel_type(&self) -> &str {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(f) => PanelFactory::panel_type(f),
            #[cfg(feature = "doom")]
            Self::DoomStats(f) => PanelFactory::panel_type(f),
            Self::Metrics(f) => PanelFactory::panel_type(f),
            Self::Proprioception(f) => PanelFactory::panel_type(f),
            #[cfg(test)]
            Self::TestMock(f) => PanelFactory::panel_type(f),
            #[cfg(test)]
            Self::TestFailing(f) => PanelFactory::panel_type(f),
        }
    }

    fn create(&self, config: &CustomPanelConfig) -> Result<PanelInstanceImpl> {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(f) => PanelFactory::create(f, config),
            #[cfg(feature = "doom")]
            Self::DoomStats(f) => PanelFactory::create(f, config),
            Self::Metrics(f) => PanelFactory::create(f, config),
            Self::Proprioception(f) => PanelFactory::create(f, config),
            #[cfg(test)]
            Self::TestMock(f) => PanelFactory::create(f, config),
            #[cfg(test)]
            Self::TestFailing(f) => PanelFactory::create(f, config),
        }
    }

    fn description(&self) -> &'static str {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(f) => PanelFactory::description(f),
            #[cfg(feature = "doom")]
            Self::DoomStats(f) => PanelFactory::description(f),
            Self::Metrics(f) => PanelFactory::description(f),
            Self::Proprioception(f) => PanelFactory::description(f),
            #[cfg(test)]
            Self::TestMock(f) => PanelFactory::description(f),
            #[cfg(test)]
            Self::TestFailing(f) => PanelFactory::description(f),
        }
    }
}
