// SPDX-License-Identifier: AGPL-3.0-or-later
//! Rich TUI application struct and main loop

use crate::error::TuiError;
use crate::events::EventHandler;
use crate::state::TUIState;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use super::config::TUIConfig;
use super::render::render_current_view;
use super::update::handle_event;

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
    ///
    /// # Errors
    /// Returns `TuiError` on terminal setup failure.
    pub async fn new() -> Result<Self, TuiError> {
        Self::with_config(TUIConfig::default()).await
    }

    /// Create new TUI with custom config
    ///
    /// # Errors
    /// Returns `TuiError` on terminal setup failure.
    pub async fn with_config(config: TUIConfig) -> Result<Self, TuiError> {
        // Setup terminal
        enable_raw_mode().map_err(|e| TuiError::terminal("Failed to enable raw mode", e))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)
            .map_err(|e| TuiError::terminal("Failed to enter alternate screen", e))?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)
            .map_err(|e| TuiError::terminal("Failed to create terminal", e))?;

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
    ///
    /// # Errors
    /// Returns `TuiError` on render or event handling failure.
    pub async fn run(&mut self) -> Result<(), TuiError> {
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
                handle_event(self, event).await?;
            }
        }

        // Cleanup
        self.cleanup()?;

        Ok(())
    }

    /// Render current view
    pub(super) async fn render(&mut self) -> Result<(), TuiError> {
        let state = self.state.clone();
        let view = state.get_view().await;

        self.terminal
            .draw(|f| render_current_view(f, &state, view))
            .map_err(|e| TuiError::terminal("Failed to draw", e))?;

        Ok(())
    }

    /// Cleanup terminal
    fn cleanup(&mut self) -> Result<(), TuiError> {
        disable_raw_mode().map_err(|e| TuiError::terminal("Failed to disable raw mode", e))?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)
            .map_err(|e| TuiError::terminal("Failed to leave alternate screen", e))?;
        self.terminal
            .show_cursor()
            .map_err(|e| TuiError::terminal("Failed to show cursor", e))?;

        Ok(())
    }

    /// Access state for update module
    pub(super) const fn state(&self) -> &TUIState {
        &self.state
    }

    /// Access state mutably for update module
    #[allow(clippy::missing_const_for_fn)]
    pub(super) fn state_mut(&mut self) -> &mut TUIState {
        &mut self.state
    }

    /// Set running flag for update module
    #[allow(clippy::missing_const_for_fn)]
    pub(super) fn set_running(&mut self, running: bool) {
        self.running = running;
    }

    /// Discover primals (called from run, needs to be accessible from update for refresh)
    pub(super) async fn discover_primals(&mut self) -> Result<(), TuiError> {
        super::update::discover_primals(self).await
    }
}

// Ensure cleanup on drop
impl Drop for RichTUI {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
