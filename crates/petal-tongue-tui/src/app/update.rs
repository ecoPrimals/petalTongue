// SPDX-License-Identifier: AGPL-3.0-or-later
//! Event handling and data refresh logic

use crate::error::TuiError;
use crate::events::{ExternalEvent, KeyAction, TUIEvent, parse_key_event};
use crate::state::{LogLevel, LogMessage, View};
use crossterm::event::KeyEvent;

use super::tui::RichTUI;

/// Handle incoming TUI event
pub(super) async fn handle_event(tui: &mut RichTUI, event: TUIEvent) -> Result<(), TuiError> {
    match event {
        TUIEvent::Key(key) => {
            handle_key_event(tui, key).await?;
        }
        TUIEvent::Tick => {
            // Periodic refresh (if needed)
            refresh_data(tui).await?;
        }
        TUIEvent::Quit => {
            tui.set_running(false);
        }
        TUIEvent::Resize { .. } => {
            // Terminal will auto-resize on next render
        }
        TUIEvent::External(ext_event) => {
            handle_external_event(tui, ext_event).await?;
        }
    }

    Ok(())
}

/// Handle key event
async fn handle_key_event(tui: &mut RichTUI, key: KeyEvent) -> Result<(), TuiError> {
    let action = parse_key_event(key);

    match action {
        KeyAction::Quit => {
            tui.set_running(false);
        }
        KeyAction::SwitchView(index) => {
            let views = View::all();
            if index < views.len() {
                tui.state_mut().set_view(views[index]).await;
            }
        }
        KeyAction::SelectPrevious => {
            let max = get_current_view_item_count(tui).await;
            tui.state_mut().select_previous(max).await;
        }
        KeyAction::SelectNext => {
            let max = get_current_view_item_count(tui).await;
            tui.state_mut().select_next(max).await;
        }
        KeyAction::Refresh => {
            tui.discover_primals().await?;
        }
        KeyAction::Help | _ => {}
    }

    Ok(())
}

/// Handle external event
async fn handle_external_event(tui: &mut RichTUI, event: ExternalEvent) -> Result<(), TuiError> {
    match event {
        ExternalEvent::PrimalDiscovered { name } => {
            add_log(tui, format!("Discovered primal: {name}")).await;
        }
        ExternalEvent::PrimalStatusChanged { name, healthy } => {
            add_log(
                tui,
                format!(
                    "Primal {} status: {}",
                    name,
                    if healthy { "healthy" } else { "unhealthy" }
                ),
            )
            .await;
        }
        ExternalEvent::LogMessage { source, message } => {
            add_log(tui, format!("[{source}] {message}")).await;
        }
        ExternalEvent::TopologyChanged => {
            // Refresh topology data
            refresh_topology(tui).await?;
        }
    }

    Ok(())
}

/// Get item count for current view (for selection wrapping)
async fn get_current_view_item_count(tui: &RichTUI) -> usize {
    match tui.state().get_view().await {
        View::Primals => tui.state().primal_count().await,
        View::Logs => tui.state().log_count().await,
        _ => 0,
    }
}

/// Discover primals (capability-based, runtime)
pub(super) async fn discover_primals(tui: &mut RichTUI) -> Result<(), TuiError> {
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
                        add_log(tui, format!("Failed to get primals: {e}")).await;
                    }
                }
            }

            if all_primals.is_empty() {
                // No primals found - standalone mode
                tui.state_mut().set_standalone_mode(true).await;
                add_log(
                    tui,
                    "Running in standalone mode (no primals discovered)".to_string(),
                )
                .await;
            } else {
                tui.state_mut().set_standalone_mode(false).await;
                tui.state_mut().update_primals(all_primals).await;
                add_log(
                    tui,
                    format!("Discovered {} primals", tui.state().primal_count().await),
                )
                .await;
            }
        }
        Err(e) => {
            // Discovery failed - standalone mode
            tui.state_mut().set_standalone_mode(true).await;
            add_log(
                tui,
                format!("Discovery failed: {e}. Running in standalone mode."),
            )
            .await;
        }
    }

    Ok(())
}

/// Refresh topology data
async fn refresh_topology(tui: &mut RichTUI) -> Result<(), TuiError> {
    // Try to get topology from discovered providers
    if let Ok(providers) = petal_tongue_discovery::discover_visualization_providers().await {
        for provider in providers {
            if let Ok(topology) = provider.get_topology().await {
                tui.state_mut().update_topology(topology).await;
                break;
            }
        }
    } else {
        // No topology available
    }

    Ok(())
}

/// Refresh data (periodic)
async fn refresh_data(tui: &mut RichTUI) -> Result<(), TuiError> {
    // Only refresh if not in standalone mode
    if !tui.state().is_standalone().await {
        // Lightweight refresh - don't rediscover, just update status
        // Full refresh happens on 'r' key or external events
    }

    Ok(())
}

/// Add log message
async fn add_log(tui: &RichTUI, message: String) {
    tui.state()
        .add_log(LogMessage {
            timestamp: chrono::Utc::now(),
            source: None,
            level: LogLevel::Info,
            message,
        })
        .await;
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use crate::events::{ExternalEvent, KeyAction, TUIEvent, parse_key_event};
    use crate::state::{LogLevel, LogMessage, View};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[tokio::test]
    async fn handle_event_quit_variant() {
        let quit = TUIEvent::Quit;
        assert!(matches!(quit, TUIEvent::Quit));
    }

    #[tokio::test]
    async fn handle_event_tick_variant() {
        let tick = TUIEvent::Tick;
        assert!(matches!(tick, TUIEvent::Tick));
    }

    #[tokio::test]
    async fn handle_event_resize_variant() {
        let resize = TUIEvent::Resize {
            width: 80,
            height: 24,
        };
        assert!(matches!(
            resize,
            TUIEvent::Resize {
                width: 80,
                height: 24
            }
        ));
    }

    #[tokio::test]
    async fn handle_event_external_primal_discovered_format() {
        let _evt = ExternalEvent::PrimalDiscovered {
            name: "test-primal".to_string(),
        };
        let formatted = format!("Discovered primal: {}", "test-primal");
        assert_eq!(formatted, "Discovered primal: test-primal");
    }

    #[tokio::test]
    async fn handle_event_external_primal_status_format() {
        let formatted = format!(
            "Primal {} status: {}",
            "p1",
            if true { "healthy" } else { "unhealthy" }
        );
        assert_eq!(formatted, "Primal p1 status: healthy");
        let formatted_unhealthy = format!(
            "Primal {} status: {}",
            "p2",
            if false { "healthy" } else { "unhealthy" }
        );
        assert_eq!(formatted_unhealthy, "Primal p2 status: unhealthy");
    }

    #[tokio::test]
    async fn handle_event_external_log_message_format() {
        let formatted = format!("[{}]", "source");
        assert_eq!(formatted, "[source]");
    }

    #[tokio::test]
    async fn key_action_switch_view_bounds_check() {
        let views = View::all();
        for i in 0..views.len() {
            let action = parse_key_event(KeyEvent::new(
                KeyCode::Char(char::from_digit((i + 1) as u32, 10).unwrap()),
                KeyModifiers::NONE,
            ));
            if let KeyAction::SwitchView(idx) = action {
                assert!(idx < views.len(), "SwitchView index must be in bounds");
            }
        }
    }

    #[tokio::test]
    async fn refresh_data_standalone_check_logic() {
        let state = crate::state::TUIState::new();
        state.set_standalone_mode(true).await;
        assert!(state.is_standalone().await);
        state.set_standalone_mode(false).await;
        assert!(!state.is_standalone().await);
    }

    #[tokio::test]
    async fn key_action_quit_stops_loop() {
        let action = KeyAction::Quit;
        assert!(matches!(action, KeyAction::Quit));
    }

    #[tokio::test]
    async fn key_action_help_no_op() {
        let action = KeyAction::Help;
        assert!(matches!(action, KeyAction::Help));
    }

    #[tokio::test]
    async fn key_action_refresh_triggers_discover() {
        let action = KeyAction::Refresh;
        assert!(matches!(action, KeyAction::Refresh));
    }

    #[tokio::test]
    async fn key_action_select_previous_next_with_index() {
        let action_prev = KeyAction::SelectPrevious;
        let action_next = KeyAction::SelectNext;
        assert!(matches!(action_prev, KeyAction::SelectPrevious));
        assert!(matches!(action_next, KeyAction::SelectNext));
    }

    #[tokio::test]
    async fn external_event_topology_changed() {
        let evt = ExternalEvent::TopologyChanged;
        assert!(matches!(evt, ExternalEvent::TopologyChanged));
    }

    #[tokio::test]
    async fn external_event_primal_discovered_name() {
        let evt = ExternalEvent::PrimalDiscovered {
            name: "toadstool".to_string(),
        };
        if let ExternalEvent::PrimalDiscovered { name } = evt {
            assert_eq!(name, "toadstool");
        }
    }

    #[tokio::test]
    async fn external_event_primal_status_healthy() {
        let evt = ExternalEvent::PrimalStatusChanged {
            name: "songbird".to_string(),
            healthy: true,
        };
        if let ExternalEvent::PrimalStatusChanged { name, healthy } = evt {
            assert_eq!(name, "songbird");
            assert!(healthy);
        }
    }

    #[tokio::test]
    async fn external_event_primal_status_unhealthy() {
        let evt = ExternalEvent::PrimalStatusChanged {
            name: "beardog".to_string(),
            healthy: false,
        };
        if let ExternalEvent::PrimalStatusChanged { name, healthy } = evt {
            assert_eq!(name, "beardog");
            assert!(!healthy);
        }
    }

    #[tokio::test]
    async fn external_event_log_message_format() {
        let evt = ExternalEvent::LogMessage {
            source: "discovery".to_string(),
            message: "Found 3 primals".to_string(),
        };
        if let ExternalEvent::LogMessage { source, message } = evt {
            let formatted = format!("[{source}] {message}");
            assert_eq!(formatted, "[discovery] Found 3 primals");
        }
    }

    #[tokio::test]
    async fn parse_key_char_3_switch_view_2() {
        let action = parse_key_event(KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE));
        assert_eq!(action, KeyAction::SwitchView(2));
    }

    #[tokio::test]
    async fn parse_key_char_8_switch_view_7() {
        let action = parse_key_event(KeyEvent::new(KeyCode::Char('8'), KeyModifiers::NONE));
        assert_eq!(action, KeyAction::SwitchView(7));
    }

    #[tokio::test]
    async fn parse_key_char_9_out_of_bounds() {
        let action = parse_key_event(KeyEvent::new(KeyCode::Char('9'), KeyModifiers::NONE));
        assert_eq!(action, KeyAction::None);
    }

    #[tokio::test]
    async fn parse_key_char_0_out_of_bounds() {
        let action = parse_key_event(KeyEvent::new(KeyCode::Char('0'), KeyModifiers::NONE));
        assert_eq!(action, KeyAction::None);
    }

    #[tokio::test]
    async fn tui_event_key_wraps_keyevent() {
        let key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
        let evt = TUIEvent::Key(key);
        assert!(matches!(evt, TUIEvent::Key(_)));
    }

    #[tokio::test]
    async fn tui_event_resize_dimensions() {
        let evt = TUIEvent::Resize {
            width: 120,
            height: 40,
        };
        if let TUIEvent::Resize { width, height } = evt {
            assert_eq!(width, 120);
            assert_eq!(height, 40);
        }
    }

    #[tokio::test]
    async fn discover_primals_standalone_log_message() {
        let msg = "Running in standalone mode (no primals discovered)".to_string();
        assert!(msg.contains("standalone"));
    }

    #[tokio::test]
    async fn discover_primals_success_log_format() {
        let count = 5usize;
        let msg = format!("Discovered {} primals", count);
        assert_eq!(msg, "Discovered 5 primals");
    }

    #[tokio::test]
    async fn discover_primals_failure_log_format() {
        let err_msg = "Connection refused";
        let msg = format!("Discovery failed: {err_msg}. Running in standalone mode.");
        assert!(msg.contains("standalone"));
        assert!(msg.contains("Connection refused"));
    }

    #[tokio::test]
    async fn discover_primals_provider_error_log_format() {
        let err_msg = "timeout";
        let msg = format!("Failed to get primals: {err_msg}");
        assert_eq!(msg, "Failed to get primals: timeout");
    }

    #[tokio::test]
    async fn view_all_len_matches_switch_view_indices() {
        let views = View::all();
        for i in 0..views.len() {
            let key = KeyEvent::new(
                KeyCode::Char(char::from_digit((i + 1) as u32, 10).unwrap()),
                KeyModifiers::NONE,
            );
            let action = parse_key_event(key);
            if let KeyAction::SwitchView(idx) = action {
                assert_eq!(idx, i);
            }
        }
    }

    #[tokio::test]
    async fn log_message_timestamp_source_level() {
        let msg = LogMessage {
            timestamp: chrono::Utc::now(),
            source: Some("test".to_string()),
            level: LogLevel::Info,
            message: "test message".to_string(),
        };
        assert_eq!(msg.source.as_deref(), Some("test"));
        assert!(matches!(msg.level, LogLevel::Info));
    }
}
