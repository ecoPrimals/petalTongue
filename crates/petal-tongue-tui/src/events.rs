// SPDX-License-Identifier: AGPL-3.0-only
//! Event System
//!
//! Handles keyboard, mouse, and async events for the TUI.
//! Pure Rust, zero unsafe code.

use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use tokio::sync::mpsc;

/// TUI event
#[derive(Debug, Clone)]
pub enum TUIEvent {
    /// Keyboard input
    Key(KeyEvent),

    /// Tick event (for animation/refresh)
    Tick,

    /// Quit application
    Quit,

    /// Resize event
    Resize {
        /// New terminal width
        width: u16,
        /// New terminal height
        height: u16,
    },

    /// External event (from other primals)
    External(ExternalEvent),
}

/// External event from other primals
#[derive(Debug, Clone)]
pub enum ExternalEvent {
    /// Primal discovered
    PrimalDiscovered {
        /// Name of discovered primal
        name: String,
    },

    /// Primal status changed
    PrimalStatusChanged {
        /// Name of primal
        name: String,
        /// Whether primal is healthy
        healthy: bool,
    },

    /// New log message
    LogMessage {
        /// Source of log message
        source: String,
        /// Log message content
        message: String,
    },

    /// Topology changed
    TopologyChanged,
}

/// Event handler
///
/// Manages event loop and dispatches events.
/// Zero unsafe code, pure async.
pub struct EventHandler {
    /// Event sender
    tx: mpsc::UnboundedSender<TUIEvent>,

    /// Event receiver
    rx: mpsc::UnboundedReceiver<TUIEvent>,

    /// Tick rate
    tick_rate: Duration,
}

impl EventHandler {
    /// Create new event handler
    #[must_use]
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self { tx, rx, tick_rate }
    }

    /// Get event sender (for external events)
    #[must_use]
    pub fn sender(&self) -> mpsc::UnboundedSender<TUIEvent> {
        self.tx.clone()
    }

    /// Start event loop
    ///
    /// Spawns a background task to read terminal events
    pub fn start(&self) {
        let tx = self.tx.clone();
        let tick_rate = self.tick_rate;

        tokio::spawn(async move {
            let mut tick_interval = tokio::time::interval(tick_rate);

            loop {
                // Wait for either a terminal event or a tick
                tokio::select! {
                    _ = tick_interval.tick() => {
                        if tx.send(TUIEvent::Tick).is_err() {
                            break;
                        }
                    }
                    event = Self::read_terminal_event() => {
                        match event {
                            Ok(Some(evt)) => {
                                if tx.send(evt).is_err() {
                                    break;
                                }
                            }
                            Ok(None) => {}
                            Err(_) => break,
                        }
                    }
                }
            }
        });
    }

    /// Read terminal event (async)
    #[expect(
        clippy::unused_async,
        reason = "async for future non-blocking crossterm API"
    )]
    async fn read_terminal_event() -> Result<Option<TUIEvent>> {
        // Poll for event with timeout
        if crossterm::event::poll(Duration::from_millis(100))? {
            match crossterm::event::read()? {
                CrosstermEvent::Key(key) => Ok(Some(TUIEvent::Key(key))),
                CrosstermEvent::Resize(width, height) => {
                    Ok(Some(TUIEvent::Resize { width, height }))
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Receive next event
    pub async fn next(&mut self) -> Option<TUIEvent> {
        self.rx.recv().await
    }

    /// Send external event
    pub fn send_external(&self, event: ExternalEvent) -> Result<()> {
        self.tx.send(TUIEvent::External(event))?;
        Ok(())
    }
}

/// Parse key event into action
#[must_use]
pub const fn parse_key_event(key: KeyEvent) -> KeyAction {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), KeyModifiers::NONE) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            KeyAction::Quit
        }

        // View switching (1-8)
        (KeyCode::Char('1'), KeyModifiers::NONE) => KeyAction::SwitchView(0),
        (KeyCode::Char('2'), KeyModifiers::NONE) => KeyAction::SwitchView(1),
        (KeyCode::Char('3'), KeyModifiers::NONE) => KeyAction::SwitchView(2),
        (KeyCode::Char('4'), KeyModifiers::NONE) => KeyAction::SwitchView(3),
        (KeyCode::Char('5'), KeyModifiers::NONE) => KeyAction::SwitchView(4),
        (KeyCode::Char('6'), KeyModifiers::NONE) => KeyAction::SwitchView(5),
        (KeyCode::Char('7'), KeyModifiers::NONE) => KeyAction::SwitchView(6),
        (KeyCode::Char('8'), KeyModifiers::NONE) => KeyAction::SwitchView(7),

        (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => KeyAction::SelectPrevious,
        (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => KeyAction::SelectNext,
        (KeyCode::Left, _) | (KeyCode::Char('h'), KeyModifiers::NONE) => KeyAction::ScrollLeft,
        (KeyCode::Right, _) | (KeyCode::Char('l'), KeyModifiers::NONE) => KeyAction::ScrollRight,

        // Page navigation
        (KeyCode::PageUp, _) => KeyAction::PageUp,
        (KeyCode::PageDown, _) => KeyAction::PageDown,
        (KeyCode::Home, _) => KeyAction::Home,
        (KeyCode::End, _) => KeyAction::End,

        // Actions
        (KeyCode::Enter, _) => KeyAction::Select,
        (KeyCode::Esc, _) => KeyAction::Back,
        (KeyCode::Char(' '), KeyModifiers::NONE) => KeyAction::Toggle,

        // Help
        (KeyCode::Char('?'), KeyModifiers::NONE) => KeyAction::Help,

        // Refresh
        (KeyCode::Char('r'), KeyModifiers::NONE) => KeyAction::Refresh,

        // Unknown
        _ => KeyAction::None,
    }
}

/// Key action
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAction {
    /// Quit application
    Quit,

    /// Switch to view (index)
    SwitchView(usize),

    /// Select previous item
    SelectPrevious,

    /// Select next item
    SelectNext,

    /// Scroll left
    ScrollLeft,

    /// Scroll right
    ScrollRight,

    /// Page up
    PageUp,

    /// Page down
    PageDown,

    /// Go to top
    Home,

    /// Go to bottom
    End,

    /// Select/activate current item
    Select,

    /// Go back/cancel
    Back,

    /// Toggle current item
    Toggle,

    /// Show help
    Help,

    /// Refresh data
    Refresh,

    /// No action
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_quit_keys() {
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
            KeyAction::Quit
        );
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            KeyAction::Quit
        );
    }

    #[test]
    fn test_parse_view_switching() {
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE)),
            KeyAction::SwitchView(0)
        );
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::Char('8'), KeyModifiers::NONE)),
            KeyAction::SwitchView(7)
        );
    }

    #[test]
    fn test_parse_navigation() {
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
            KeyAction::SelectPrevious
        );
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            KeyAction::SelectNext
        );
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE)),
            KeyAction::SelectPrevious
        );
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)),
            KeyAction::SelectNext
        );
    }

    #[test]
    fn test_parse_actions() {
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            KeyAction::Select
        );
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            KeyAction::Back
        );
        assert_eq!(
            parse_key_event(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)),
            KeyAction::Toggle
        );
    }

    #[tokio::test]
    async fn test_event_handler() {
        let mut handler = EventHandler::new(Duration::from_millis(100));
        handler.start();

        // Send external event
        handler
            .send_external(ExternalEvent::PrimalDiscovered {
                name: "songbird".to_string(),
            })
            .unwrap();

        // Receive event
        if let Some(TUIEvent::External(ExternalEvent::PrimalDiscovered { name })) =
            handler.next().await
        {
            assert_eq!(name, "songbird");
        } else {
            panic!("Expected external event");
        }
    }

    #[test]
    fn test_event_handler_new() {
        let handler = EventHandler::new(Duration::from_secs(1));
        let sender = handler.sender();
        assert!(!sender.is_closed());
    }

    #[test]
    fn test_key_action_debug_eq() {
        assert_eq!(KeyAction::ScrollLeft, KeyAction::ScrollLeft);
        assert_ne!(KeyAction::ScrollLeft, KeyAction::ScrollRight);
        assert_eq!(KeyAction::Toggle, KeyAction::Toggle);
    }
}
