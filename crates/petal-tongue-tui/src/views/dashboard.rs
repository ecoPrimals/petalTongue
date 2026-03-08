// SPDX-License-Identifier: AGPL-3.0-only
//! Dashboard View
//!
//! System overview showing primals, topology, and status.
//! Leverages Songbird if available, graceful degradation to standalone.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::state::TUIState;
use crate::widgets::StatusBar;

/// Render dashboard view
pub fn render(frame: &mut Frame, area: Rect, state: &TUIState) {
    // Split into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Status summary
            Constraint::Min(0),    // Main content (2 columns)
            Constraint::Length(3), // Bottom status bar
        ])
        .split(area);

    // Render status summary
    render_status_summary(frame, chunks[0], state);

    // Split main content into columns
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Left: Primals
            Constraint::Percentage(50), // Right: Topology summary
        ])
        .split(chunks[1]);

    // Render primal list
    render_primal_list(frame, main_chunks[0], state);

    // Render topology summary
    render_topology_summary(frame, main_chunks[1], state);

    // Render bottom status bar
    render_bottom_status(frame, chunks[2], state);
}

/// Render status summary at top
fn render_status_summary(frame: &mut Frame, area: Rect, state: &TUIState) {
    let stats = tokio::runtime::Handle::current().block_on(state.stats());
    let status = tokio::runtime::Handle::current().block_on(state.get_status());

    let lines = vec![
        Line::from(vec![
            Span::styled("🌸 ", Style::default().fg(Color::Magenta)),
            Span::styled(
                "petalTongue Dashboard",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Active Primals: "),
            Span::styled(
                format!("{}", status.active_primals),
                Style::default().fg(if status.active_primals > 0 {
                    Color::Green
                } else {
                    Color::Yellow
                }),
            ),
            Span::raw("  |  Topology Edges: "),
            Span::styled(
                format!("{}", stats.topology_edge_count),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw("  |  Logs: "),
            Span::styled(
                format!("{}", stats.log_count),
                Style::default().fg(Color::Gray),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(Block::default().borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}

/// Render primal list (left column)
fn render_primal_list(frame: &mut Frame, area: Rect, state: &TUIState) {
    let primals = tokio::runtime::Handle::current().block_on(state.get_primals());
    let standalone = tokio::runtime::Handle::current().block_on(state.is_standalone());

    let items: Vec<ListItem> = if standalone {
        vec![ListItem::new(Line::from(vec![Span::styled(
            "⚠️  Standalone Mode",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]))]
    } else if primals.is_empty() {
        vec![ListItem::new(Line::from(vec![Span::styled(
            "🔍 Discovering primals...",
            Style::default().fg(Color::Gray),
        )]))]
    } else {
        primals
            .iter()
            .map(|primal| {
                use petal_tongue_core::PrimalHealthStatus;

                let (health_icon, health_color, health_text) = match primal.health {
                    PrimalHealthStatus::Healthy => ("✅", Color::Green, "Healthy"),
                    PrimalHealthStatus::Warning => ("⚠️", Color::Yellow, "Warning"),
                    PrimalHealthStatus::Critical => ("❌", Color::Red, "Critical"),
                    PrimalHealthStatus::Unknown => ("❓", Color::Gray, "Unknown"),
                };

                ListItem::new(Line::from(vec![
                    Span::raw(format!("{health_icon} ")),
                    Span::styled(&primal.name, Style::default().fg(Color::Cyan)),
                    Span::raw(" ("),
                    Span::styled(&primal.primal_type, Style::default().fg(Color::Magenta)),
                    Span::raw(") - "),
                    Span::styled(health_text, Style::default().fg(health_color)),
                ]))
            })
            .collect()
    };

    let list = List::new(items).block(
        Block::default()
            .title("🌸 Primals")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(list, area);
}

/// Render topology summary (right column)
fn render_topology_summary(frame: &mut Frame, area: Rect, state: &TUIState) {
    let topology = tokio::runtime::Handle::current().block_on(state.get_topology());
    let primals = tokio::runtime::Handle::current().block_on(state.get_primals());
    let standalone = tokio::runtime::Handle::current().block_on(state.is_standalone());

    let content = if standalone {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Running in standalone mode.",
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(""),
            Line::from("No other primals discovered."),
            Line::from(""),
            Line::from(vec![Span::styled(
                "💡 Tip:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("Start other primals to see them here!"),
            Line::from("Press 'r' to refresh discovery."),
        ]
    } else if topology.is_empty() && primals.is_empty() {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "🔍 Discovering topology...",
                Style::default().fg(Color::Gray),
            )]),
            Line::from(""),
            Line::from("Press 'r' to refresh."),
        ]
    } else if topology.is_empty() {
        vec![
            Line::from(""),
            Line::from(vec![Span::raw(format!(
                "Discovered {} primals",
                primals.len()
            ))]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "No topology edges yet.",
                Style::default().fg(Color::Gray),
            )]),
            Line::from(""),
            Line::from("Edges will appear as primals"),
            Line::from("establish connections."),
        ]
    } else {
        let mut lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("Nodes: "),
                Span::styled(
                    format!("{}", primals.len()),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::raw("Edges: "),
                Span::styled(
                    format!("{}", topology.len()),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Recent Connections:",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
        ];

        // Show last 5 edges
        for edge in topology.iter().take(5) {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(&edge.from, Style::default().fg(Color::Cyan)),
                Span::raw(" → "),
                Span::styled(&edge.to, Style::default().fg(Color::Magenta)),
                Span::raw(" ("),
                Span::styled(&edge.edge_type, Style::default().fg(Color::Gray)),
                Span::raw(")"),
            ]));
        }

        if topology.len() > 5 {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                format!("... and {} more", topology.len() - 5),
                Style::default().fg(Color::DarkGray),
            )]));
        }

        lines
    };

    let paragraph = Paragraph::new(content).block(
        Block::default()
            .title("📊 Topology")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(paragraph, area);
}

/// Render bottom status bar
fn render_bottom_status(frame: &mut Frame, area: Rect, state: &TUIState) {
    let status = tokio::runtime::Handle::current().block_on(state.get_status());
    StatusBar::render(frame, area, &status);
}
