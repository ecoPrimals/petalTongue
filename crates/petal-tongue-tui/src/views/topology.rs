// SPDX-License-Identifier: AGPL-3.0-only
//! Topology View
//!
//! ASCII art graph visualization of primal connections.
//! Leverages Songbird if available, ToadStool for layout compute (optional).

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::state::TUIState;

/// Render topology view
pub fn render(frame: &mut Frame, area: Rect, state: &TUIState) {
    let standalone = tokio::runtime::Handle::current().block_on(state.is_standalone());
    let primals = tokio::runtime::Handle::current().block_on(state.get_primals());
    let topology = tokio::runtime::Handle::current().block_on(state.get_topology());

    if standalone {
        render_standalone_message(frame, area);
        return;
    }

    // Split into graph area and details
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // Graph visualization
            Constraint::Percentage(30), // Node/edge details
        ])
        .split(area);

    // Render graph
    render_graph(frame, chunks[0], &primals, &topology);

    // Render details
    render_details(frame, chunks[1], &primals, &topology);
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
        Line::from("No topology available in standalone mode."),
        Line::from(""),
        Line::from(vec![Span::styled(
            "💡 Tip:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("Start other primals to visualize their connections!"),
        Line::from(""),
        Line::from("Press 'r' to refresh discovery."),
        Line::from("Press '1' to return to Dashboard."),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title("📊 Topology")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(paragraph, area);
}

/// Render ASCII art graph
fn render_graph(
    frame: &mut Frame,
    area: Rect,
    primals: &[petal_tongue_core::PrimalInfo],
    topology: &[petal_tongue_core::TopologyEdge],
) {
    let lines = if primals.is_empty() {
        vec![
            Line::from(""),
            Line::from(""),
            Line::from(vec![Span::styled(
                "🔍 No primals discovered yet...",
                Style::default().fg(Color::Gray),
            )]),
            Line::from(""),
            Line::from("Press 'r' to refresh discovery."),
        ]
    } else if topology.is_empty() {
        // Show primals without connections
        let mut lines = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Discovered Primals:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
        ];

        use petal_tongue_core::PrimalHealthStatus;

        for primal in primals {
            let health_icon = match primal.health {
                PrimalHealthStatus::Healthy => "✅",
                PrimalHealthStatus::Warning => "⚠️",
                PrimalHealthStatus::Critical => "❌",
                PrimalHealthStatus::Unknown => "❓",
            };

            lines.push(Line::from(vec![Span::raw("    ┌─────────────┐")]));
            lines.push(Line::from(vec![
                Span::raw("    │ "),
                Span::styled(&primal.name, Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::raw(health_icon),
                Span::raw(" │"),
            ]));
            lines.push(Line::from(vec![Span::raw("    └─────────────┘")]));
            lines.push(Line::from(""));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "No connections established yet.",
            Style::default().fg(Color::Gray),
        )]));

        lines
    } else {
        // Render graph with connections (simplified ASCII art)
        render_ascii_graph(primals, topology)
    };

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title("📊 Topology Graph")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(paragraph, area);
}

/// Render ASCII graph with connections
fn render_ascii_graph<'a>(
    primals: &'a [petal_tongue_core::PrimalInfo],
    topology: &'a [petal_tongue_core::TopologyEdge],
) -> Vec<Line<'a>> {
    let mut lines = vec![Line::from("")];

    // Simple vertical layout for now
    // TODO: Implement proper force-directed layout (optionally with ToadStool)

    use petal_tongue_core::PrimalHealthStatus;

    for (i, primal) in primals.iter().enumerate() {
        let health_icon = match primal.health {
            PrimalHealthStatus::Healthy => "✅",
            PrimalHealthStatus::Warning => "⚠️",
            PrimalHealthStatus::Critical => "❌",
            PrimalHealthStatus::Unknown => "❓",
        };

        // Node box
        lines.push(Line::from(vec![Span::raw("    ┌─────────────────┐")]));
        lines.push(Line::from(vec![
            Span::raw("    │ "),
            Span::styled(
                format!("{} {} ", health_icon, &primal.name),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" │"),
        ]));
        lines.push(Line::from(vec![
            Span::raw("    │ ("),
            Span::styled(&primal.primal_type, Style::default().fg(Color::Magenta)),
            Span::raw(")      │"),
        ]));
        lines.push(Line::from(vec![Span::raw("    └─────────────────┘")]));

        // Show outgoing edges
        let outgoing: Vec<_> = topology.iter().filter(|e| e.from == primal.id).collect();

        for edge in outgoing {
            lines.push(Line::from(vec![Span::raw("           │")]));
            lines.push(Line::from(vec![
                Span::raw("           ↓ "),
                Span::styled(&edge.edge_type, Style::default().fg(Color::Yellow)),
            ]));
        }

        if i < primals.len() - 1 {
            lines.push(Line::from(""));
        }
    }

    lines
}

/// Render details sidebar
fn render_details(
    frame: &mut Frame,
    area: Rect,
    primals: &[petal_tongue_core::PrimalInfo],
    topology: &[petal_tongue_core::TopologyEdge],
) {
    let items: Vec<ListItem> = if primals.is_empty() {
        vec![ListItem::new(Line::from(vec![Span::styled(
            "No primals discovered",
            Style::default().fg(Color::Gray),
        )]))]
    } else {
        let mut items = vec![
            ListItem::new(Line::from(vec![Span::styled(
                "📊 Graph Statistics:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![
                Span::raw("Nodes: "),
                Span::styled(
                    format!("{}", primals.len()),
                    Style::default().fg(Color::Green),
                ),
            ])),
            ListItem::new(Line::from(vec![
                Span::raw("Edges: "),
                Span::styled(
                    format!("{}", topology.len()),
                    Style::default().fg(Color::Cyan),
                ),
            ])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "Edge Types:",
                Style::default().add_modifier(Modifier::BOLD),
            )])),
            ListItem::new(Line::from("")),
        ];

        // Count edge types
        let mut edge_types: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for edge in topology {
            *edge_types.entry(edge.edge_type.clone()).or_insert(0) += 1;
        }

        for (edge_type, count) in edge_types {
            items.push(ListItem::new(Line::from(vec![
                Span::raw("  • "),
                Span::styled(edge_type, Style::default().fg(Color::Yellow)),
                Span::raw(": "),
                Span::styled(format!("{count}"), Style::default().fg(Color::Green)),
            ])));
        }

        items
    };

    let list = List::new(items).block(
        Block::default()
            .title("Details")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(list, area);
}
