// SPDX-License-Identifier: AGPL-3.0-or-later
//! Devices View
//!
//! Device management and assignment.
//! Leverages discovery provider for device discovery.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::state::TUIState;

#[must_use]
pub fn format_device_entry(index: usize, status: &str) -> (String, String) {
    (format!("Device {index}"), status.to_string())
}

#[must_use]
pub fn format_device_count_display(count: usize) -> String {
    format!("Discovered {count} devices:")
}

/// Render devices view
pub fn render(frame: &mut Frame, area: Rect, state: &TUIState) {
    let standalone = tokio::runtime::Handle::current().block_on(state.is_standalone());
    let status = tokio::runtime::Handle::current().block_on(state.get_status());

    if standalone {
        render_standalone_message(frame, area);
        return;
    }

    // Split into device list and details
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Device list
            Constraint::Percentage(40), // Device details
        ])
        .split(area);

    // Render device list
    render_device_list(frame, chunks[0], &status);

    // Render device details
    render_device_details(frame, chunks[1], &status);
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

/// Render device list
fn render_device_list(frame: &mut Frame, area: Rect, status: &crate::state::SystemStatus) {
    let device_count = status.discovered_devices;

    let items: Vec<ListItem> = if device_count == 0 {
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
        // Show discovered devices
        let mut items = vec![
            ListItem::new(Line::from(vec![Span::styled(
                format_device_count_display(device_count),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )])),
            ListItem::new(Line::from("")),
        ];

        // Placeholder device entries
        for i in 1..=device_count.min(10) {
            let (name, status) = format_device_entry(i, "Available");
            items.push(ListItem::new(Line::from(vec![
                Span::styled("📱 ", Style::default().fg(Color::Cyan)),
                Span::raw(name),
                Span::raw(" ("),
                Span::styled(status, Style::default().fg(Color::Green)),
                Span::raw(")"),
            ])));
        }

        if device_count > 10 {
            items.push(ListItem::new(Line::from("")));
            items.push(ListItem::new(Line::from(vec![Span::styled(
                format!("... and {} more", device_count - 10),
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

/// Render device details
fn render_device_details(frame: &mut Frame, area: Rect, status: &crate::state::SystemStatus) {
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
                format!("{}", status.discovered_devices),
                Style::default().fg(Color::Green),
            ),
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
        Line::from(""),
        Line::from(vec![Span::styled(
            "💡 Note:",
            Style::default().fg(Color::Yellow),
        )]),
        Line::from("Device assignment requires"),
        Line::from("integration with the discovery provider."),
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

    #[test]
    fn format_device_entry_first() {
        let (name, status) = format_device_entry(1, "Available");
        assert_eq!(name, "Device 1");
        assert_eq!(status, "Available");
    }

    #[test]
    fn format_device_entry_tenth() {
        let (name, status) = format_device_entry(10, "Offline");
        assert_eq!(name, "Device 10");
        assert_eq!(status, "Offline");
    }

    #[test]
    fn format_device_count_zero() {
        assert_eq!(format_device_count_display(0), "Discovered 0 devices:");
    }

    #[test]
    fn format_device_count_many() {
        assert_eq!(format_device_count_display(42), "Discovered 42 devices:");
    }
}
