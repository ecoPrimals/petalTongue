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
            tick_rate: Duration::from_millis(100),
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

    #[test]
    fn tui_config_default_values() {
        let config = TUIConfig::default();
        assert_eq!(config.tick_rate, Duration::from_millis(100));
        assert!(!config.mouse_support);
        assert!(!config.standalone);
    }

    #[test]
    fn view_all_returns_all_views() {
        let views = crate::state::View::all();
        assert_eq!(views.len(), 8);
        assert_eq!(views[0].shortcut(), '1');
        assert_eq!(views[0].name(), "Dashboard");
        assert_eq!(views[0], crate::state::View::Dashboard);
    }
}
