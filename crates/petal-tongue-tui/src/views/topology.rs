// SPDX-License-Identifier: AGPL-3.0-or-later
//! Topology View
//!
//! ASCII art graph visualization of primal connections.
//! Leverages discovery provider if available, layout compute optional.

use std::collections::HashMap;

use petal_tongue_core::{PrimalId, PrimalInfo, TopologyEdge};

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::state::TUIState;

const fn health_icon_for_status(health: petal_tongue_core::PrimalHealthStatus) -> &'static str {
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

/// Fruchterman-Reingold force-directed layout
///
/// - Repulsive force between all node pairs: Fr = k²/d
/// - Attractive force along edges: Fa = d²/k
/// - k = `sqrt(area/num_nodes)`
/// - Iterates with cooling temperature
fn force_directed_layout(
    primals: &[PrimalInfo],
    topology: &[TopologyEdge],
    width: f64,
    height: f64,
    iterations: usize,
) -> HashMap<PrimalId, (f64, f64)> {
    let n = primals.len();
    if n == 0 {
        return HashMap::new();
    }

    let area = width * height;
    #[expect(
        clippy::cast_precision_loss,
        reason = "layout calculation, precision sufficient"
    )]
    let k = (area / n as f64).sqrt().max(1.0);
    let mut temperature = (area / 10.0).sqrt();

    let id_to_idx: HashMap<&str, usize> = primals
        .iter()
        .enumerate()
        .map(|(i, p)| (p.id.as_str(), i))
        .collect();

    // Initial positions (circle)
    let mut positions: Vec<(f64, f64)> = (0..n)
        .map(|i| {
            #[expect(
                clippy::cast_precision_loss,
                reason = "circular layout, precision sufficient"
            )]
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
            (
                (width / 4.0).mul_add(angle.cos(), width / 2.0),
                (height / 4.0).mul_add(angle.sin(), height / 2.0),
            )
        })
        .collect();

    let mut disp: Vec<(f64, f64)> = vec![(0.0, 0.0); n];

    for _ in 0..iterations {
        disp.fill((0.0, 0.0));

        // Repulsive: Fr = k²/d
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = positions[i].0 - positions[j].0;
                let dy = positions[i].1 - positions[j].1;
                let d = dx.hypot(dy).max(0.01);
                let fr = k * k / d;
                let fx = (dx / d) * fr;
                let fy = (dy / d) * fr;
                disp[i].0 += fx;
                disp[i].1 += fy;
                disp[j].0 -= fx;
                disp[j].1 -= fy;
            }
        }

        // Attractive along edges: Fa = d²/k
        for edge in topology {
            if let (Some(&i), Some(&j)) = (
                id_to_idx.get(edge.from.as_str()),
                id_to_idx.get(edge.to.as_str()),
            ) && i < n
                && j < n
            {
                let dx = positions[i].0 - positions[j].0;
                let dy = positions[i].1 - positions[j].1;
                let d = dx.hypot(dy).max(0.01);
                let fa = d * d / k;
                let fx = (dx / d) * fa;
                let fy = (dy / d) * fa;
                disp[i].0 -= fx;
                disp[i].1 -= fy;
                disp[j].0 += fx;
                disp[j].1 += fy;
            }
        }

        // Apply displacement with cooling
        for i in 0..n {
            let (dx, dy) = disp[i];
            let len = dx.hypot(dy).max(0.01);
            let lim = len.min(temperature);
            positions[i].0 += (dx / len) * lim;
            positions[i].1 += (dy / len) * lim;
            // Keep in bounds
            positions[i].0 = positions[i].0.clamp(0.0, width);
            positions[i].1 = positions[i].1.clamp(0.0, height);
        }

        temperature *= 0.95;
    }

    primals
        .iter()
        .enumerate()
        .map(|(i, p)| (p.id.clone(), positions[i]))
        .collect()
}

/// Render ASCII graph with force-directed layout
pub fn render_ascii_graph<'a>(
    primals: &'a [PrimalInfo],
    topology: &'a [TopologyEdge],
) -> Vec<Line<'a>> {
    // Terminal dimensions for layout (chars)
    const LAYOUT_WIDTH: f64 = 70.0;
    const LAYOUT_HEIGHT: f64 = 20.0;

    let mut lines = vec![Line::from("")];

    if primals.is_empty() {
        return lines;
    }

    let positions = force_directed_layout(primals, topology, LAYOUT_WIDTH, LAYOUT_HEIGHT, 50);

    // Sort nodes by position (row then col) for layout order, then render
    let mut ordered: Vec<_> = primals
        .iter()
        .enumerate()
        .filter_map(|(idx, p)| positions.get(&p.id).map(|&pos| (pos, idx, p)))
        .collect();
    ordered.sort_by(|a, b| {
        a.0.1
            .partial_cmp(&b.0.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.0.0
                    .partial_cmp(&b.0.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    for (_, _, primal) in ordered {
        let health_icon = health_icon_for_status(primal.health);

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

        let outgoing: Vec<_> = topology.iter().filter(|e| e.from == primal.id).collect();
        for edge in outgoing {
            lines.push(Line::from(vec![Span::raw("           │")]));
            lines.push(Line::from(vec![
                Span::raw("           ↓ "),
                Span::styled(&edge.edge_type, Style::default().fg(Color::Yellow)),
            ]));
        }

        lines.push(Line::from(""));
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
                Span::styled(primals.len().to_string(), Style::default().fg(Color::Green)),
            ])),
            ListItem::new(Line::from(vec![
                Span::raw("Edges: "),
                Span::styled(topology.len().to_string(), Style::default().fg(Color::Cyan)),
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
#[path = "topology_tests.rs"]
mod tests;
