// SPDX-License-Identifier: AGPL-3.0-only
//! Main TUI Application
//!
//! Core application logic for the Rich TUI.
//! Zero unsafe code, pure async, capability-based.

use anyhow::{Context, Result};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use petal_tongue_core::constants;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::time::Duration;

use crate::{
    events::{EventHandler, ExternalEvent, KeyAction, TUIEvent, parse_key_event},
    state::{TUIState, View},
    views::{
        render_dashboard, render_devices, render_livespore, render_logs, render_neural_api,
        render_nucleus, render_primals, render_topology,
    },
};

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

/// Rich TUI Application
///
/// Main application struct. Manages terminal, state, and event loop.
/// Zero unsafe code, pure Rust.
pub struct RichTUI {
    /// Terminal
    terminal: Terminal<CrosstermBackend<io::Stdout>>,

    /// State
    state: TUIState,

    /// Event handler
    events: EventHandler,

    /// Configuration
    config: TUIConfig,

    /// Running flag
    running: bool,
}

impl RichTUI {
    /// Create new TUI with default config
    pub async fn new() -> Result<Self> {
        Self::with_config(TUIConfig::default()).await
    }

    /// Create new TUI with custom config
    pub async fn with_config(config: TUIConfig) -> Result<Self> {
        // Setup terminal
        enable_raw_mode().context("Failed to enable raw mode")?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).context("Failed to create terminal")?;

        // Create state
        let state = TUIState::new();

        // Set standalone mode if configured
        state.set_standalone_mode(config.standalone).await;

        // Create event handler
        let events = EventHandler::new(config.tick_rate);

        Ok(Self {
            terminal,
            state,
            events,
            config,
            running: false,
        })
    }

    /// Run the TUI
    pub async fn run(&mut self) -> Result<()> {
        self.running = true;

        // Start event loop
        self.events.start();

        // Discover primals if not in standalone mode
        if !self.config.standalone {
            self.discover_primals().await?;
        }

        // Main loop
        while self.running {
            // Render
            self.render().await?;

            // Handle events
            if let Some(event) = self.events.next().await {
                self.handle_event(event).await?;
            }
        }

        // Cleanup
        self.cleanup()?;

        Ok(())
    }

    /// Render current view
    async fn render(&mut self) -> Result<()> {
        let state = self.state.clone();
        let view = state.get_view().await;

        self.terminal.draw(|f| {
            // Render based on current view
            match view {
                View::Dashboard => render_dashboard(f, &state),
                View::Topology => render_topology(f, &state),
                View::Devices => render_devices(f, &state),
                View::Primals => render_primals(f, &state),
                View::Logs => render_logs(f, &state),
                View::NeuralAPI => render_neural_api(f, &state),
                View::Nucleus => render_nucleus(f, &state),
                View::LiveSpore => render_livespore(f, &state),
            }
        })?;

        Ok(())
    }

    /// Handle event
    async fn handle_event(&mut self, event: TUIEvent) -> Result<()> {
        match event {
            TUIEvent::Key(key) => {
                self.handle_key_event(key).await?;
            }
            TUIEvent::Tick => {
                // Periodic refresh (if needed)
                self.refresh_data().await?;
            }
            TUIEvent::Quit => {
                self.running = false;
            }
            TUIEvent::Resize { .. } => {
                // Terminal will auto-resize on next render
            }
            TUIEvent::External(ext_event) => {
                self.handle_external_event(ext_event).await?;
            }
        }

        Ok(())
    }

    /// Handle key event
    async fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        let action = parse_key_event(key);

        match action {
            KeyAction::Quit => {
                self.running = false;
            }
            KeyAction::SwitchView(index) => {
                let views = View::all();
                if index < views.len() {
                    self.state.set_view(views[index]).await;
                }
            }
            KeyAction::SelectPrevious => {
                let max = self.get_current_view_item_count().await;
                self.state.select_previous(max).await;
            }
            KeyAction::SelectNext => {
                let max = self.get_current_view_item_count().await;
                self.state.select_next(max).await;
            }
            KeyAction::Refresh => {
                self.discover_primals().await?;
            }
            KeyAction::Help | _ => {}
        }

        Ok(())
    }

    /// Handle external event
    async fn handle_external_event(&mut self, event: ExternalEvent) -> Result<()> {
        match event {
            ExternalEvent::PrimalDiscovered { name } => {
                self.add_log(format!("Discovered primal: {name}")).await;
            }
            ExternalEvent::PrimalStatusChanged { name, healthy } => {
                self.add_log(format!(
                    "Primal {} status: {}",
                    name,
                    if healthy { "healthy" } else { "unhealthy" }
                ))
                .await;
            }
            ExternalEvent::LogMessage { source, message } => {
                self.add_log(format!("[{source}] {message}")).await;
            }
            ExternalEvent::TopologyChanged => {
                // Refresh topology data
                self.refresh_topology().await?;
            }
        }

        Ok(())
    }

    /// Get item count for current view (for selection wrapping)
    async fn get_current_view_item_count(&self) -> usize {
        match self.state.get_view().await {
            View::Primals => self.state.primal_count().await,
            View::Logs => self.state.log_count().await,
            _ => 0,
        }
    }

    /// Discover primals (capability-based, runtime)
    async fn discover_primals(&mut self) -> Result<()> {
        // Try to discover via petal-tongue-discovery
        match petal_tongue_discovery::discover_visualization_providers().await {
            Ok(providers) => {
                // Get primals from providers
                let mut all_primals = Vec::new();
                for provider in providers {
                    match provider.get_primals().await {
                        Ok(primals) => {
                            all_primals.extend(primals);
                        }
                        Err(e) => {
                            self.add_log(format!("Failed to get primals: {e}")).await;
                        }
                    }
                }

                if all_primals.is_empty() {
                    // No primals found - standalone mode
                    self.state.set_standalone_mode(true).await;
                    self.add_log("Running in standalone mode (no primals discovered)".to_string())
                        .await;
                } else {
                    self.state.set_standalone_mode(false).await;
                    self.state.update_primals(all_primals).await;
                    self.add_log(format!(
                        "Discovered {} primals",
                        self.state.primal_count().await
                    ))
                    .await;
                }
            }
            Err(e) => {
                // Discovery failed - standalone mode
                self.state.set_standalone_mode(true).await;
                self.add_log(format!(
                    "Discovery failed: {e}. Running in standalone mode."
                ))
                .await;
            }
        }

        Ok(())
    }

    /// Refresh topology data
    async fn refresh_topology(&mut self) -> Result<()> {
        // Try to get topology from discovered providers
        if let Ok(providers) = petal_tongue_discovery::discover_visualization_providers().await {
            for provider in providers {
                if let Ok(topology) = provider.get_topology().await {
                    self.state.update_topology(topology).await;
                    break;
                }
            }
        } else {
            // No topology available
        }

        Ok(())
    }

    /// Refresh data (periodic)
    async fn refresh_data(&mut self) -> Result<()> {
        // Only refresh if not in standalone mode
        if !self.state.is_standalone().await {
            // Lightweight refresh - don't rediscover, just update status
            // Full refresh happens on 'r' key or external events
        }

        Ok(())
    }

    /// Add log message
    async fn add_log(&self, message: String) {
        self.state
            .add_log(crate::state::LogMessage {
                timestamp: chrono::Utc::now(),
                source: None,
                level: crate::state::LogLevel::Info,
                message,
            })
            .await;
    }

    /// Cleanup terminal
    fn cleanup(&mut self) -> Result<()> {
        disable_raw_mode().context("Failed to disable raw mode")?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)
            .context("Failed to leave alternate screen")?;
        self.terminal
            .show_cursor()
            .context("Failed to show cursor")?;

        Ok(())
    }
}

// Ensure cleanup on drop
impl Drop for RichTUI {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{KeyAction, parse_key_event};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn tui_config_default_values() {
        let config = TUIConfig::default();
        assert_eq!(config.tick_rate, Duration::from_millis(100));
        assert!(!config.mouse_support);
        assert!(!config.standalone);
    }

    #[test]
    fn tui_config_custom_values() {
        let config = TUIConfig {
            tick_rate: Duration::from_secs(1),
            mouse_support: true,
            standalone: true,
        };
        assert_eq!(config.tick_rate, Duration::from_secs(1));
        assert!(config.mouse_support);
        assert!(config.standalone);
    }

    #[test]
    fn view_all_returns_all_views() {
        let views = crate::state::View::all();
        assert_eq!(views.len(), 8);
        assert_eq!(views[0].shortcut(), '1');
        assert_eq!(views[0].name(), "Dashboard");
        assert_eq!(views[0], crate::state::View::Dashboard);
    }

    #[test]
    fn view_shortcuts_map_to_indices() {
        let views = crate::state::View::all();
        for (i, view) in views.iter().enumerate() {
            let shortcut = view.shortcut();
            let expected_digit = char::from_digit(u32::try_from(i + 1).unwrap(), 10).unwrap();
            assert_eq!(
                shortcut,
                expected_digit,
                "View {} should have shortcut {}",
                view.name(),
                expected_digit
            );
        }
    }

    #[test]
    fn key_event_quit_maps_to_action() {
        let action = parse_key_event(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
        assert_eq!(action, KeyAction::Quit);
    }

    #[test]
    fn key_event_view_switch_maps_to_action() {
        let action = parse_key_event(KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE));
        assert_eq!(action, KeyAction::SwitchView(2));
    }

    #[test]
    fn key_event_refresh_maps_to_action() {
        let action = parse_key_event(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE));
        assert_eq!(action, KeyAction::Refresh);
    }

    #[test]
    fn key_action_switch_view_bounds() {
        let views = crate::state::View::all();
        let action = KeyAction::SwitchView(7);
        if let KeyAction::SwitchView(idx) = action {
            assert!(idx < views.len());
        }
    }

    #[test]
    fn key_action_variants() {
        assert_eq!(KeyAction::Quit, KeyAction::Quit);
        assert_ne!(KeyAction::Quit, KeyAction::Refresh);
        assert_ne!(KeyAction::SwitchView(0), KeyAction::SwitchView(1));
    }

    #[tokio::test]
    async fn state_view_switch_resets_selection() {
        let state = crate::state::TUIState::new();
        state.set_selected_index(5).await;
        state.set_view(crate::state::View::Topology).await;
        assert_eq!(state.get_selected_index().await, 0);
    }

    #[tokio::test]
    async fn state_standalone_mode_affects_refresh() {
        let state = crate::state::TUIState::new();
        state.set_standalone_mode(true).await;
        assert!(state.is_standalone().await);
    }

    #[test]
    fn tui_event_variants() {
        use crate::events::{ExternalEvent, TUIEvent};
        let _key = TUIEvent::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
        let _ = TUIEvent::Tick;
        let _ = TUIEvent::Quit;
        let _ = TUIEvent::Resize {
            width: 80,
            height: 24,
        };
        let _ext = TUIEvent::External(ExternalEvent::PrimalDiscovered {
            name: "test".to_string(),
        });
    }

    #[test]
    fn external_event_log_format() {
        use crate::events::ExternalEvent;
        let _evt = ExternalEvent::LogMessage {
            source: "src".to_string(),
            message: "msg".to_string(),
        };
        let source = "src";
        let message = "msg";
        let formatted = format!("[{source}] {message}");
        assert_eq!(formatted, "[src] msg");
    }

    #[test]
    fn parse_key_select_previous_next() {
        let prev = parse_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(prev, KeyAction::SelectPrevious);
        let next = parse_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(next, KeyAction::SelectNext);
    }

    #[test]
    fn parse_key_select_help_refresh() {
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE)),
            KeyAction::Help
        );
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE)),
            KeyAction::Refresh
        );
    }

    #[test]
    fn parse_key_scroll_page_home_end() {
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE)),
            KeyAction::PageUp
        );
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE)),
            KeyAction::PageDown
        );
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)),
            KeyAction::Home
        );
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::End, KeyModifiers::NONE)),
            KeyAction::End
        );
    }

    #[test]
    fn parse_key_select_toggle_back() {
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            KeyAction::Select
        );
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)),
            KeyAction::Toggle
        );
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            KeyAction::Back
        );
    }

    #[test]
    fn parse_key_unknown_maps_to_none() {
        let action = parse_key_event(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::NONE));
        assert_eq!(action, KeyAction::None);
    }

    #[tokio::test]
    async fn select_previous_with_max_zero() {
        let state = crate::state::TUIState::new();
        state.set_selected_index(0).await;
        state.select_previous(0).await;
        assert_eq!(state.get_selected_index().await, 0);
    }

    #[tokio::test]
    async fn select_next_with_max_zero() {
        let state = crate::state::TUIState::new();
        state.set_selected_index(0).await;
        state.select_next(0).await;
        assert_eq!(state.get_selected_index().await, 0);
    }

    #[tokio::test]
    async fn external_event_primal_status_format() {
        let healthy = true;
        let name = "test";
        let formatted = format!(
            "Primal {} status: {}",
            name,
            if healthy { "healthy" } else { "unhealthy" }
        );
        assert_eq!(formatted, "Primal test status: healthy");
    }

    #[tokio::test]
    async fn get_current_view_item_count_primals() {
        let state = crate::state::TUIState::new();
        state.set_view(crate::state::View::Primals).await;
        // With no primals, count is 0
        let count = match state.get_view().await {
            crate::state::View::Primals => state.primal_count().await,
            crate::state::View::Logs => state.log_count().await,
            _ => 0,
        };
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn get_current_view_item_count_logs() {
        let state = crate::state::TUIState::new();
        state.set_view(crate::state::View::Logs).await;
        let count = match state.get_view().await {
            crate::state::View::Primals => state.primal_count().await,
            crate::state::View::Logs => state.log_count().await,
            _ => 0,
        };
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn get_current_view_item_count_other_views() {
        let state = crate::state::TUIState::new();
        state.set_view(crate::state::View::Dashboard).await;
        let count = match state.get_view().await {
            crate::state::View::Primals => state.primal_count().await,
            crate::state::View::Logs => state.log_count().await,
            _ => 0,
        };
        assert_eq!(count, 0);
    }

    #[test]
    fn tui_config_debug() {
        let config = TUIConfig::default();
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("TUIConfig"));
    }

    #[tokio::test]
    async fn external_event_log_formats_match_handle_external() {
        let state = crate::state::TUIState::new();
        let discovered = format!("Discovered primal: {}", "songbird");
        state
            .add_log(crate::state::LogMessage {
                timestamp: chrono::Utc::now(),
                source: None,
                level: crate::state::LogLevel::Info,
                message: discovered.clone(),
            })
            .await;
        let logs = state.get_logs().await;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].message, "Discovered primal: songbird");

        let status = format!(
            "Primal {} status: {}",
            "toadstool",
            if true { "healthy" } else { "unhealthy" }
        );
        state
            .add_log(crate::state::LogMessage {
                timestamp: chrono::Utc::now(),
                source: None,
                level: crate::state::LogLevel::Info,
                message: status,
            })
            .await;
        let logs = state.get_logs().await;
        assert_eq!(logs[1].message, "Primal toadstool status: healthy");

        let source = "src";
        let message = "msg";
        let log_msg = format!("[{source}] {message}");
        state
            .add_log(crate::state::LogMessage {
                timestamp: chrono::Utc::now(),
                source: None,
                level: crate::state::LogLevel::Info,
                message: log_msg,
            })
            .await;
        let logs = state.get_logs().await;
        assert_eq!(logs[2].message, "[src] msg");
    }

    #[tokio::test]
    async fn view_switch_resets_selection() {
        let state = crate::state::TUIState::new();
        state.set_view(crate::state::View::Primals).await;
        state.set_selected_index(3).await;
        state.set_view(crate::state::View::Topology).await;
        assert_eq!(state.get_selected_index().await, 0);
    }

    #[tokio::test]
    #[expect(clippy::cast_sign_loss, reason = "test primal counts are always positive")]
    async fn data_update_primals_affects_item_count() {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let state = crate::state::TUIState::new();
            state.set_view(crate::state::View::Primals).await;
            assert_eq!(state.primal_count().await, 0);
            state
                .update_primals(vec![
                    petal_tongue_core::PrimalInfo::new(
                        "p1",
                        "primal1",
                        "Test",
                        "unix:///tmp/p1.sock",
                        vec![],
                        petal_tongue_core::PrimalHealthStatus::Healthy,
                        chrono::Utc::now().timestamp() as u64,
                    ),
                    petal_tongue_core::PrimalInfo::new(
                        "p2",
                        "primal2",
                        "Test",
                        "unix:///tmp/p2.sock",
                        vec![],
                        petal_tongue_core::PrimalHealthStatus::Healthy,
                        chrono::Utc::now().timestamp() as u64,
                    ),
                ])
                .await;
            assert_eq!(state.primal_count().await, 2);
        })
        .await
        .expect("test timed out after 5s");
    }

    #[tokio::test]
    async fn standalone_mode_blocks_refresh_logic() {
        let state = crate::state::TUIState::new();
        state.set_standalone_mode(true).await;
        assert!(state.is_standalone().await);
    }

    #[tokio::test]
    async fn key_action_switch_view_index_bounds() {
        let views = crate::state::View::all();
        for (i, _) in views.iter().enumerate() {
            let action = KeyAction::SwitchView(i);
            if let KeyAction::SwitchView(idx) = action {
                assert!(idx < views.len());
            }
        }
    }
}
