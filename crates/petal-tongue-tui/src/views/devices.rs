// SPDX-License-Identifier: AGPL-3.0-or-later
//! Devices View
//!
//! Device management and assignment.
//! Renders discovered primals from the discovery provider — no placeholder data.

use petal_tongue_core::{PrimalHealthStatus, PrimalInfo};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::sync::Arc;

use crate::state::TUIState;

#[must_use]
pub fn format_device_count_display(count: usize) -> String {
    format!("Discovered {count} devices:")
}

const fn health_color(health: &PrimalHealthStatus) -> Color {
    match health {
        PrimalHealthStatus::Healthy => Color::Green,
        PrimalHealthStatus::Warning => Color::Yellow,
        PrimalHealthStatus::Critical => Color::Red,
        PrimalHealthStatus::Unknown => Color::DarkGray,
    }
}

const fn health_label(health: &PrimalHealthStatus) -> &'static str {
    match health {
        PrimalHealthStatus::Healthy => "Healthy",
        PrimalHealthStatus::Warning => "Warning",
        PrimalHealthStatus::Critical => "Critical",
        PrimalHealthStatus::Unknown => "Unknown",
    }
}

/// Render devices view
pub fn render(frame: &mut Frame, area: Rect, state: &TUIState) {
    let standalone = tokio::runtime::Handle::current().block_on(state.is_standalone());

    if standalone {
        render_standalone_message(frame, area);
        return;
    }

    let primals = tokio::runtime::Handle::current().block_on(state.get_primals());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    render_device_list(frame, chunks[0], &primals);
    render_device_details(frame, chunks[1], &primals);
}

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
        Line::from("Device discovery requires the discovery provider."),
        Line::from(""),
        Line::from(vec![Span::styled(
            "💡 Tip:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("Start the discovery provider to discover devices."),
        Line::from(""),
        Line::from("Press 'r' to refresh discovery."),
        Line::from("Press '1' to return to Dashboard."),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title("📱 Devices")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(paragraph, area);
}

fn render_device_list(frame: &mut Frame, area: Rect, primals: &Arc<Vec<PrimalInfo>>) {
    let items: Vec<ListItem> = if primals.is_empty() {
        vec![
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "🔍 No devices discovered yet...",
                Style::default().fg(Color::Gray),
            )])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from("Devices will appear here once")),
            ListItem::new(Line::from("discovered by the discovery provider.")),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from("Press 'r' to refresh.")),
        ]
    } else {
        let mut items = vec![
            ListItem::new(Line::from(vec![Span::styled(
                format_device_count_display(primals.len()),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )])),
            ListItem::new(Line::from("")),
        ];

        let max_display = 10;
        for primal in primals.iter().take(max_display) {
            let color = health_color(&primal.health);
            let status = health_label(&primal.health);
            items.push(ListItem::new(Line::from(vec![
                Span::styled("📱 ", Style::default().fg(Color::Cyan)),
                Span::raw(&primal.name),
                Span::styled(
                    format!(" [{type}]", type = primal.primal_type),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(" ("),
                Span::styled(status, Style::default().fg(color)),
                Span::raw(")"),
            ])));
        }

        if primals.len() > max_display {
            items.push(ListItem::new(Line::from("")));
            items.push(ListItem::new(Line::from(vec![Span::styled(
                format!("... and {} more", primals.len() - max_display),
                Style::default().fg(Color::DarkGray),
            )])));
        }

        items
    };

    let list = List::new(items).block(
        Block::default()
            .title("📱 Devices")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(list, area);
}

fn render_device_details(frame: &mut Frame, area: Rect, primals: &Arc<Vec<PrimalInfo>>) {
    let cap_count: usize = primals.iter().map(|p| p.capabilities.len()).sum();

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Device Management",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Total Devices: "),
            Span::styled(
                format!("{}", primals.len()),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::raw("Total Capabilities: "),
            Span::styled(format!("{cap_count}"), Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Actions:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("  [Enter] Assign device"),
        Line::from("  [d] Device details"),
        Line::from("  [r] Refresh discovery"),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title("Details")
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
    fn format_device_count_zero() {
        assert_eq!(format_device_count_display(0), "Discovered 0 devices:");
    }

    #[test]
    fn format_device_count_many() {
        assert_eq!(format_device_count_display(42), "Discovered 42 devices:");
    }

    #[test]
    fn health_color_healthy_is_green() {
        assert_eq!(health_color(&PrimalHealthStatus::Healthy), Color::Green);
    }

    #[test]
    fn health_color_critical_is_red() {
        assert_eq!(health_color(&PrimalHealthStatus::Critical), Color::Red);
    }

    #[test]
    fn health_label_covers_all_variants() {
        assert_eq!(health_label(&PrimalHealthStatus::Healthy), "Healthy");
        assert_eq!(health_label(&PrimalHealthStatus::Warning), "Warning");
        assert_eq!(health_label(&PrimalHealthStatus::Critical), "Critical");
        assert_eq!(health_label(&PrimalHealthStatus::Unknown), "Unknown");
    }
}
