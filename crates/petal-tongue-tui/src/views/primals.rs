// SPDX-License-Identifier: AGPL-3.0-only
//! Primals View
//!
//! Detailed primal status and health monitoring.
//! Leverages discovery provider for primal discovery.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::state::TUIState;
use petal_tongue_core::PrimalHealthStatus;

#[must_use]
pub const fn format_primal_health_list(health: PrimalHealthStatus) -> (&'static str, Color) {
    match health {
        PrimalHealthStatus::Healthy => ("✅", Color::Green),
        PrimalHealthStatus::Warning => ("⚠️", Color::Yellow),
        PrimalHealthStatus::Critical => ("❌", Color::Red),
        PrimalHealthStatus::Unknown => ("❓", Color::Gray),
    }
}

#[must_use]
pub const fn format_primal_health_details(health: PrimalHealthStatus) -> (&'static str, Color) {
    match health {
        PrimalHealthStatus::Healthy => ("Healthy", Color::Green),
        PrimalHealthStatus::Warning => ("Warning", Color::Yellow),
        PrimalHealthStatus::Critical => ("Critical", Color::Red),
        PrimalHealthStatus::Unknown => ("Unknown", Color::Gray),
    }
}

#[must_use]
pub fn format_primal_list_title(count: usize) -> String {
    format!("🌸 Primals ({count} total)")
}

/// Render primals view
pub fn render(frame: &mut Frame, area: Rect, state: &TUIState) {
    let standalone = tokio::runtime::Handle::current().block_on(state.is_standalone());
    let primals = tokio::runtime::Handle::current().block_on(state.get_primals());
    let selected = tokio::runtime::Handle::current().block_on(state.get_selected_index());

    if standalone {
        render_standalone_message(frame, area);
        return;
    }

    // Split into primal list and details
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Primal list
            Constraint::Percentage(50), // Primal details
        ])
        .split(area);

    // Render primal list
    render_primal_list(frame, chunks[0], &primals, selected);

    // Render primal details
    render_primal_details(frame, chunks[1], &primals, selected);
}

/// Render standalone message
fn render_standalone_message(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "⚠️  Standalone Mode",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("No primals discovered."),
        Line::from(""),
        Line::from(vec![Span::styled(
            "💡 Tip:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("Start other primals to see them here!"),
        Line::from(""),
        Line::from("Press 'r' to refresh discovery."),
        Line::from("Press '1' to return to Dashboard."),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title("🌸 Primals")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(paragraph, area);
}

/// Render primal list
fn render_primal_list(
    frame: &mut Frame,
    area: Rect,
    primals: &[petal_tongue_core::PrimalInfo],
    selected: usize,
) {
    let items: Vec<ListItem> = if primals.is_empty() {
        vec![
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "🔍 Discovering primals...",
                Style::default().fg(Color::Gray),
            )])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from("Press 'r' to refresh.")),
        ]
    } else {
        primals
            .iter()
            .enumerate()
            .map(|(idx, primal)| {
                let (health_icon, health_color) = format_primal_health_list(primal.health);

                let is_selected = idx == selected;
                let style = if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{health_icon} "), style.fg(health_color)),
                    Span::styled(&primal.name, style.fg(Color::Cyan)),
                    Span::styled(" (", style),
                    Span::styled(&primal.primal_type, style.fg(Color::Magenta)),
                    Span::styled(")", style),
                ]))
                .style(style)
            })
            .collect()
    };

    let title = format_primal_list_title(primals.len());

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(list, area);
}

/// Render primal details
fn render_primal_details(
    frame: &mut Frame,
    area: Rect,
    primals: &[petal_tongue_core::PrimalInfo],
    selected: usize,
) {
    let lines = if primals.is_empty() {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "No primal selected",
                Style::default().fg(Color::Gray),
            )]),
        ]
    } else if selected >= primals.len() {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Invalid selection",
                Style::default().fg(Color::Red),
            )]),
        ]
    } else {
        let primal = &primals[selected];

        let (health_text, health_color) = format_primal_health_details(primal.health);

        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                &primal.name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Type: "),
                Span::styled(&primal.primal_type, Style::default().fg(Color::Magenta)),
            ]),
            Line::from(vec![
                Span::raw("Health: "),
                Span::styled(health_text, Style::default().fg(health_color)),
            ]),
            Line::from(vec![
                Span::raw("ID: "),
                Span::styled(primal.id.as_str(), Style::default().fg(Color::Gray)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Capabilities:",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from("  • Discovery"),
            Line::from("  • Monitoring"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Actions:",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from("  [Enter] View details"),
            Line::from("  [h] Health check"),
            Line::from("  [r] Refresh"),
        ]
    };

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title("Primal Details")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::PrimalHealthStatus;

    #[test]
    fn format_primal_health_list_healthy() {
        let (icon, _) = format_primal_health_list(PrimalHealthStatus::Healthy);
        assert_eq!(icon, "✅");
    }

    #[test]
    fn format_primal_health_list_warning() {
        let (icon, _) = format_primal_health_list(PrimalHealthStatus::Warning);
        assert_eq!(icon, "⚠️");
    }

    #[test]
    fn format_primal_health_list_critical() {
        let (icon, _) = format_primal_health_list(PrimalHealthStatus::Critical);
        assert_eq!(icon, "❌");
    }

    #[test]
    fn format_primal_health_list_unknown() {
        let (icon, _) = format_primal_health_list(PrimalHealthStatus::Unknown);
        assert_eq!(icon, "❓");
    }

    #[test]
    fn format_primal_health_details_healthy() {
        let (text, _) = format_primal_health_details(PrimalHealthStatus::Healthy);
        assert_eq!(text, "Healthy");
    }

    #[test]
    fn format_primal_health_details_critical() {
        let (text, _) = format_primal_health_details(PrimalHealthStatus::Critical);
        assert_eq!(text, "Critical");
    }

    #[test]
    fn format_primal_list_title_empty() {
        assert_eq!(format_primal_list_title(0), "🌸 Primals (0 total)");
    }

    #[test]
    fn format_primal_list_title_many() {
        assert_eq!(format_primal_list_title(42), "🌸 Primals (42 total)");
    }

    #[test]
    fn format_primal_health_list_colors() {
        let (_, c_healthy) = format_primal_health_list(PrimalHealthStatus::Healthy);
        let (_, c_warning) = format_primal_health_list(PrimalHealthStatus::Warning);
        let (_, c_critical) = format_primal_health_list(PrimalHealthStatus::Critical);
        let (_, c_unknown) = format_primal_health_list(PrimalHealthStatus::Unknown);
        assert_eq!(c_healthy, Color::Green);
        assert_eq!(c_warning, Color::Yellow);
        assert_eq!(c_critical, Color::Red);
        assert_eq!(c_unknown, Color::Gray);
    }

    #[test]
    fn format_primal_health_details_all_variants() {
        let (text, _) = format_primal_health_details(PrimalHealthStatus::Warning);
        assert_eq!(text, "Warning");
        let (text, _) = format_primal_health_details(PrimalHealthStatus::Unknown);
        assert_eq!(text, "Unknown");
    }
}
