// SPDX-License-Identifier: AGPL-3.0-or-later
//! Live IPC visualization sessions bridge.
//!
//! Polls `VisualizationState` (shared with the IPC server) for active sessions
//! and presents them as live panels in the UI. This closes the IPC-to-UI bridge:
//! when an external primal calls `visualization.render`, the data appears here.

use petal_tongue_ipc::visualization_handler::VisualizationState;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Summary of an active IPC session, extracted as pure data for rendering.
#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub session_id: String,
    pub title: String,
    pub binding_count: usize,
    pub domain: Option<String>,
    pub age_secs: f32,
}

/// Extract summaries of all active sessions from the shared state.
///
/// Pure function: reads from `VisualizationState`, returns data-only summaries.
pub fn active_session_summaries(
    viz_state: &Arc<RwLock<VisualizationState>>,
) -> Vec<SessionSummary> {
    let Ok(state) = viz_state.read() else {
        return Vec::new();
    };
    let now = Instant::now();
    state
        .sessions()
        .iter()
        .map(|(id, session)| SessionSummary {
            session_id: id.clone(),
            title: session.title.clone(),
            binding_count: session.bindings.len(),
            domain: session.domain.clone(),
            age_secs: now
                .checked_duration_since(session.updated_at)
                .map_or(0.0, |d| d.as_secs_f32()),
        })
        .collect()
}

/// Check whether any session has been updated since the given timestamp.
pub fn has_updates_since(viz_state: &Arc<RwLock<VisualizationState>>, since: Instant) -> bool {
    let Ok(state) = viz_state.read() else {
        return false;
    };
    state
        .sessions()
        .values()
        .any(|session| session.updated_at > since)
}

/// Format domain as a display label.
#[must_use]
pub fn domain_label(domain: &Option<String>) -> &str {
    domain.as_deref().unwrap_or("general")
}

/// Color hint for a domain (r, g, b).
#[must_use]
pub fn domain_color_rgb(domain: &Option<String>) -> (u8, u8, u8) {
    match domain.as_deref() {
        Some("health") => (100, 180, 200),
        Some("physics") => (180, 120, 220),
        Some("ecology") => (120, 170, 100),
        Some("agriculture") => (180, 160, 100),
        Some("measurement") => (150, 150, 160),
        Some("neural") => (140, 120, 200),
        Some("game") => (220, 160, 80),
        _ => (160, 160, 170),
    }
}

/// Format a session age for display.
#[must_use]
pub fn format_session_age(age_secs: f32) -> String {
    if age_secs < 1.0 {
        "just now".to_string()
    } else if age_secs < 60.0 {
        format!("{age_secs:.0}s ago")
    } else {
        format!("{:.1}m ago", age_secs / 60.0)
    }
}

/// Render active IPC sessions panel using egui.
pub fn render_sessions_panel(ui: &mut egui::Ui, viz_state: &Arc<RwLock<VisualizationState>>) {
    let summaries = active_session_summaries(viz_state);

    if summaries.is_empty() {
        ui.label(
            egui::RichText::new("No active IPC sessions")
                .size(12.0)
                .color(egui::Color32::GRAY),
        );
        return;
    }

    ui.label(
        egui::RichText::new(format!("{} active session(s)", summaries.len()))
            .size(13.0)
            .strong(),
    );
    ui.separator();

    for summary in &summaries {
        let (r, g, b) = domain_color_rgb(&summary.domain);
        let domain_text = domain_label(&summary.domain);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("●").color(egui::Color32::from_rgb(r, g, b)));
            ui.label(egui::RichText::new(&summary.title).strong());
            ui.label(
                egui::RichText::new(format!("[{domain_text}]"))
                    .size(11.0)
                    .color(egui::Color32::GRAY),
            );
        });
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!(
                    "  {} binding(s) • {}",
                    summary.binding_count,
                    format_session_age(summary.age_secs)
                ))
                .size(11.0)
                .color(egui::Color32::GRAY),
            );
        });
        ui.add_space(4.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_label_known() {
        assert_eq!(domain_label(&Some("health".to_string())), "health");
    }

    #[test]
    fn domain_label_none() {
        assert_eq!(domain_label(&None), "general");
    }

    #[test]
    fn domain_color_known_domains() {
        assert_eq!(
            domain_color_rgb(&Some("health".to_string())),
            (100, 180, 200)
        );
        assert_eq!(domain_color_rgb(&Some("game".to_string())), (220, 160, 80));
    }

    #[test]
    fn domain_color_unknown() {
        assert_eq!(
            domain_color_rgb(&Some("unknown".to_string())),
            (160, 160, 170)
        );
        assert_eq!(domain_color_rgb(&None), (160, 160, 170));
    }

    #[test]
    fn format_session_age_just_now() {
        assert_eq!(format_session_age(0.5), "just now");
    }

    #[test]
    fn format_session_age_seconds() {
        assert_eq!(format_session_age(30.0), "30s ago");
    }

    #[test]
    fn format_session_age_minutes() {
        assert_eq!(format_session_age(120.0), "2.0m ago");
    }

    #[test]
    fn active_session_summaries_empty() {
        let state = Arc::new(RwLock::new(VisualizationState::new()));
        let summaries = active_session_summaries(&state);
        assert!(summaries.is_empty());
    }

    #[test]
    fn has_updates_since_empty() {
        let state = Arc::new(RwLock::new(VisualizationState::new()));
        assert!(!has_updates_since(&state, Instant::now()));
    }

    #[test]
    fn active_session_summaries_with_session() {
        use petal_tongue_ipc::visualization_handler::{
            VisualizationRenderRequest, VisualizationState,
        };

        let state = Arc::new(RwLock::new(VisualizationState::new()));
        {
            let mut s = state.write().expect("lock");
            s.handle_render(VisualizationRenderRequest {
                session_id: "s1".to_string(),
                title: "Test Session".to_string(),
                bindings: vec![],
                thresholds: vec![],
                domain: Some("health".to_string()),
                ui_config: None,
            });
        }
        let summaries = active_session_summaries(&state);
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].title, "Test Session");
        assert_eq!(summaries[0].domain.as_deref(), Some("health"));
    }

    #[test]
    fn has_updates_since_detects_change() {
        use petal_tongue_ipc::visualization_handler::VisualizationRenderRequest;

        let state = Arc::new(RwLock::new(VisualizationState::new()));
        let before = Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(5));
        {
            let mut s = state.write().expect("lock");
            s.handle_render(VisualizationRenderRequest {
                session_id: "s1".to_string(),
                title: "T".to_string(),
                bindings: vec![],
                thresholds: vec![],
                domain: None,
                ui_config: None,
            });
        }
        assert!(has_updates_since(&state, before));
    }
}
