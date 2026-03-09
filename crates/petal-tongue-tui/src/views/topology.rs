// SPDX-License-Identifier: AGPL-3.0-only
//! Topology View
//!
//! ASCII art graph visualization of primal connections.
//! Leverages discovery provider if available, layout compute optional.

use std::collections::HashMap;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::state::TUIState;

fn health_icon_for_status(health: petal_tongue_core::PrimalHealthStatus) -> &'static str {
    match health {
        petal_tongue_core::PrimalHealthStatus::Healthy => "✅",
        petal_tongue_core::PrimalHealthStatus::Warning => "⚠️",
        petal_tongue_core::PrimalHealthStatus::Critical => "❌",
        petal_tongue_core::PrimalHealthStatus::Unknown => "❓",
    }
}

fn count_edge_types(topology: &[petal_tongue_core::TopologyEdge]) -> HashMap<String, usize> {
    let mut edge_types = HashMap::new();
    for edge in topology {
        *edge_types.entry(edge.edge_type.clone()).or_insert(0) += 1;
    }
    edge_types
}

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

        for primal in primals {
            let health_icon = health_icon_for_status(primal.health);

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
pub(crate) fn render_ascii_graph<'a>(
    primals: &'a [petal_tongue_core::PrimalInfo],
    topology: &'a [petal_tongue_core::TopologyEdge],
) -> Vec<Line<'a>> {
    let mut lines = vec![Line::from("")];

    // Simple vertical layout for now
    // TODO: Implement proper force-directed layout (optionally with layout provider)

    for (i, primal) in primals.iter().enumerate() {
        let health_icon = health_icon_for_status(primal.health);

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

        let edge_types = count_edge_types(topology);

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

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::{PrimalHealthStatus, PrimalId, PrimalInfo, TopologyEdge};

    fn line_text(line: &ratatui::text::Line) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn render_ascii_graph_empty_primals() {
        let primals: Vec<PrimalInfo> = vec![];
        let topology: Vec<TopologyEdge> = vec![];
        let lines = render_ascii_graph(&primals, &topology);
        assert_eq!(lines.len(), 1);
        assert_eq!(line_text(&lines[0]), "");
    }

    #[test]
    fn render_ascii_graph_one_primal_no_edges() {
        let primals = vec![PrimalInfo::new(
            PrimalId::from("test-primal"),
            "TestPrimal",
            "Compute",
            "http://localhost",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        )];
        let topology: Vec<TopologyEdge> = vec![];
        let lines = render_ascii_graph(&primals, &topology);
        assert!(lines.len() >= 5);
        let all_text: String = lines.iter().map(line_text).collect::<Vec<_>>().join("\n");
        assert!(all_text.contains("TestPrimal"));
        assert!(all_text.contains("Compute"));
    }

    #[test]
    fn render_ascii_graph_with_topology_edges() {
        let primals = vec![
            PrimalInfo::new(
                PrimalId::from("a"),
                "PrimalA",
                "Compute",
                "http://a",
                vec![],
                PrimalHealthStatus::Healthy,
                0,
            ),
            PrimalInfo::new(
                PrimalId::from("b"),
                "PrimalB",
                "Storage",
                "http://b",
                vec![],
                PrimalHealthStatus::Healthy,
                0,
            ),
        ];
        let topology = vec![TopologyEdge {
            from: PrimalId::from("a"),
            to: PrimalId::from("b"),
            edge_type: "connection".to_string(),
            label: None,
            capability: None,
            metrics: None,
        }];
        let lines = render_ascii_graph(&primals, &topology);
        assert!(lines.len() >= 10);
        let all_text: String = lines.iter().map(line_text).collect::<Vec<_>>().join("\n");
        assert!(all_text.contains("PrimalA"));
        assert!(all_text.contains("PrimalB"));
        assert!(all_text.contains("connection"));
    }

    #[test]
    fn render_ascii_graph_health_icons() {
        let primals = vec![
            PrimalInfo::new(
                PrimalId::from("h"),
                "Healthy",
                "T",
                "http://h",
                vec![],
                PrimalHealthStatus::Healthy,
                0,
            ),
            PrimalInfo::new(
                PrimalId::from("w"),
                "Warning",
                "T",
                "http://w",
                vec![],
                PrimalHealthStatus::Warning,
                0,
            ),
        ];
        let topology: Vec<TopologyEdge> = vec![];
        let lines = render_ascii_graph(&primals, &topology);
        let all_text: String = lines.iter().map(line_text).collect::<Vec<_>>().join("\n");
        assert!(all_text.contains("Healthy"));
        assert!(all_text.contains("Warning"));
        assert!(all_text.contains("✅"));
        assert!(all_text.contains("⚠️"));
    }

    #[test]
    fn health_icon_for_status_mapping() {
        assert_eq!(health_icon_for_status(PrimalHealthStatus::Healthy), "✅");
        assert_eq!(health_icon_for_status(PrimalHealthStatus::Warning), "⚠️");
        assert_eq!(health_icon_for_status(PrimalHealthStatus::Critical), "❌");
        assert_eq!(health_icon_for_status(PrimalHealthStatus::Unknown), "❓");
    }

    #[test]
    fn count_edge_types_empty() {
        let topology: Vec<TopologyEdge> = vec![];
        let counts = count_edge_types(&topology);
        assert!(counts.is_empty());
    }

    #[test]
    fn count_edge_types_single_type() {
        let topology = vec![
            TopologyEdge {
                from: PrimalId::from("a"),
                to: PrimalId::from("b"),
                edge_type: "connection".to_string(),
                label: None,
                capability: None,
                metrics: None,
            },
            TopologyEdge {
                from: PrimalId::from("b"),
                to: PrimalId::from("c"),
                edge_type: "connection".to_string(),
                label: None,
                capability: None,
                metrics: None,
            },
        ];
        let counts = count_edge_types(&topology);
        assert_eq!(counts.get("connection"), Some(&2));
    }

    #[test]
    fn count_edge_types_multiple_types() {
        let topology = vec![
            TopologyEdge {
                from: PrimalId::from("a"),
                to: PrimalId::from("b"),
                edge_type: "connection".to_string(),
                label: None,
                capability: None,
                metrics: None,
            },
            TopologyEdge {
                from: PrimalId::from("b"),
                to: PrimalId::from("c"),
                edge_type: "data".to_string(),
                label: None,
                capability: None,
                metrics: None,
            },
            TopologyEdge {
                from: PrimalId::from("c"),
                to: PrimalId::from("a"),
                edge_type: "connection".to_string(),
                label: None,
                capability: None,
                metrics: None,
            },
        ];
        let counts = count_edge_types(&topology);
        assert_eq!(counts.get("connection"), Some(&2));
        assert_eq!(counts.get("data"), Some(&1));
    }
}
