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
    /// Doom game panel (feature `doom`). Boxed to reduce enum size.
    #[cfg(feature = "doom")]
    Doom(Box<crate::panels::doom_factory::DoomPanelWrapper>),
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
            Self::Doom(p) => p.render(ui),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.render(ui),
            Self::Metrics(p) => p.render(ui),
            Self::Proprioception(p) => p.render(ui),
            #[cfg(test)]
            Self::TestMock(p) => p.render(ui),
        }
    }

    fn title(&self) -> &str {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.title(),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.title(),
            Self::Metrics(p) => p.title(),
            Self::Proprioception(p) => p.title(),
            #[cfg(test)]
            Self::TestMock(p) => p.title(),
        }
    }

    fn update(&mut self) {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.update(),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.update(),
            Self::Metrics(p) => p.update(),
            Self::Proprioception(p) => p.update(),
            #[cfg(test)]
            Self::TestMock(p) => p.update(),
        }
    }

    fn on_event(&mut self, event: &egui::Event) {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.on_event(event),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.on_event(event),
            Self::Metrics(p) => p.on_event(event),
            Self::Proprioception(p) => p.on_event(event),
            #[cfg(test)]
            Self::TestMock(p) => p.on_event(event),
        }
    }

    fn wants_keyboard_input(&self) -> bool {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.wants_keyboard_input(),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.wants_keyboard_input(),
            Self::Metrics(p) => p.wants_keyboard_input(),
            Self::Proprioception(p) => p.wants_keyboard_input(),
            #[cfg(test)]
            Self::TestMock(p) => p.wants_keyboard_input(),
        }
    }

    fn wants_mouse_input(&self) -> bool {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.wants_mouse_input(),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.wants_mouse_input(),
            Self::Metrics(p) => p.wants_mouse_input(),
            Self::Proprioception(p) => p.wants_mouse_input(),
            #[cfg(test)]
            Self::TestMock(p) => p.wants_mouse_input(),
        }
    }

    fn wants_exclusive_input(&self) -> bool {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.wants_exclusive_input(),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.wants_exclusive_input(),
            Self::Metrics(p) => p.wants_exclusive_input(),
            Self::Proprioception(p) => p.wants_exclusive_input(),
            #[cfg(test)]
            Self::TestMock(p) => p.wants_exclusive_input(),
        }
    }

    fn input_priority(&self) -> u8 {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.input_priority(),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.input_priority(),
            Self::Metrics(p) => p.input_priority(),
            Self::Proprioception(p) => p.input_priority(),
            #[cfg(test)]
            Self::TestMock(p) => p.input_priority(),
        }
    }

    fn on_keyboard_event(&mut self, ctx: &egui::Context) -> crate::focus_manager::InputAction {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.on_keyboard_event(ctx),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.on_keyboard_event(ctx),
            Self::Metrics(p) => p.on_keyboard_event(ctx),
            Self::Proprioception(p) => p.on_keyboard_event(ctx),
            #[cfg(test)]
            Self::TestMock(p) => p.on_keyboard_event(ctx),
        }
    }

    fn on_mouse_event(&mut self, ctx: &egui::Context) -> crate::focus_manager::InputAction {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.on_mouse_event(ctx),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.on_mouse_event(ctx),
            Self::Metrics(p) => p.on_mouse_event(ctx),
            Self::Proprioception(p) => p.on_mouse_event(ctx),
            #[cfg(test)]
            Self::TestMock(p) => p.on_mouse_event(ctx),
        }
    }

    fn on_open(&mut self) -> crate::error::Result<()> {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.on_open(),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.on_open(),
            Self::Metrics(p) => p.on_open(),
            Self::Proprioception(p) => p.on_open(),
            #[cfg(test)]
            Self::TestMock(p) => p.on_open(),
        }
    }

    fn on_close(&mut self) -> crate::error::Result<()> {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.on_close(),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.on_close(),
            Self::Metrics(p) => p.on_close(),
            Self::Proprioception(p) => p.on_close(),
            #[cfg(test)]
            Self::TestMock(p) => p.on_close(),
        }
    }

    fn on_pause(&mut self) {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.on_pause(),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.on_pause(),
            Self::Metrics(p) => p.on_pause(),
            Self::Proprioception(p) => p.on_pause(),
            #[cfg(test)]
            Self::TestMock(p) => p.on_pause(),
        }
    }

    fn on_resume(&mut self) {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.on_resume(),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.on_resume(),
            Self::Metrics(p) => p.on_resume(),
            Self::Proprioception(p) => p.on_resume(),
            #[cfg(test)]
            Self::TestMock(p) => p.on_resume(),
        }
    }

    fn on_error(&mut self, error: &dyn std::error::Error) -> PanelAction {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.on_error(error),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.on_error(error),
            Self::Metrics(p) => p.on_error(error),
            Self::Proprioception(p) => p.on_error(error),
            #[cfg(test)]
            Self::TestMock(p) => p.on_error(error),
        }
    }

    fn can_save_state(&self) -> bool {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.can_save_state(),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.can_save_state(),
            Self::Metrics(p) => p.can_save_state(),
            Self::Proprioception(p) => p.can_save_state(),
            #[cfg(test)]
            Self::TestMock(p) => p.can_save_state(),
        }
    }

    fn can_restore_state(&self) -> bool {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.can_restore_state(),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.can_restore_state(),
            Self::Metrics(p) => p.can_restore_state(),
            Self::Proprioception(p) => p.can_restore_state(),
            #[cfg(test)]
            Self::TestMock(p) => p.can_restore_state(),
        }
    }

    fn save_state(&self) -> crate::error::Result<serde_json::Value> {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.save_state(),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.save_state(),
            Self::Metrics(p) => p.save_state(),
            Self::Proprioception(p) => p.save_state(),
            #[cfg(test)]
            Self::TestMock(p) => p.save_state(),
        }
    }

    fn restore_state(&mut self, state: serde_json::Value) -> crate::error::Result<()> {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.restore_state(state),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.restore_state(state),
            Self::Metrics(p) => p.restore_state(state),
            Self::Proprioception(p) => p.restore_state(state),
            #[cfg(test)]
            Self::TestMock(p) => p.restore_state(state),
        }
    }

    fn is_closable(&self) -> bool {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.is_closable(),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.is_closable(),
            Self::Metrics(p) => p.is_closable(),
            Self::Proprioception(p) => p.is_closable(),
            #[cfg(test)]
            Self::TestMock(p) => p.is_closable(),
        }
    }

    fn is_pausable(&self) -> bool {
        match self {
            #[cfg(feature = "doom")]
            Self::Doom(p) => p.is_pausable(),
            #[cfg(feature = "doom")]
            Self::DoomStats(p) => p.is_pausable(),
            Self::Metrics(p) => p.is_pausable(),
            Self::Proprioception(p) => p.is_pausable(),
            #[cfg(test)]
            Self::TestMock(p) => p.is_pausable(),
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
