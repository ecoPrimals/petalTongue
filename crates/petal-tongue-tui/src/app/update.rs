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
