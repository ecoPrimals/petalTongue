//! LiveSpore View
//!
//! Live deployment management for biomeOS.
//! Shows deployment pipeline and node status.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::state::TUIState;

/// Render LiveSpore view
pub fn render(frame: &mut Frame, area: Rect, state: &TUIState) {
    let standalone = tokio::runtime::Handle::current().block_on(state.is_standalone());

    // Split into deployment pipeline and node status
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Deployment pipeline
            Constraint::Percentage(50), // Node status
        ])
        .split(area);

    // Render deployment pipeline
    render_deployment_pipeline(frame, chunks[0], standalone);

    // Render node status
    render_node_status(frame, chunks[1], standalone);
}

/// Render deployment pipeline
fn render_deployment_pipeline(frame: &mut Frame, area: Rect, standalone: bool) {
    let items: Vec<ListItem> = if standalone {
        vec![
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "⚠️  Standalone Mode",
                Style::default().fg(Color::Yellow),
            )])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from("LiveSpore requires biomeOS.")),
        ]
    } else {
        vec![
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "🍄 Deployment Pipeline",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "🔍 No active deployments",
                Style::default().fg(Color::Gray),
            )])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from("Deployments will appear here")),
            ListItem::new(Line::from("when initiated via biomeOS.")),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "💡 Deployment Types:",
                Style::default().fg(Color::Cyan),
            )])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from("  • Tower (BearDog + Songbird)")),
            ListItem::new(Line::from("  • Node (Tower + ToadStool)")),
            ListItem::new(Line::from("  • Nest (Tower + NestGate)")),
            ListItem::new(Line::from("  • NUCLEUS (All atomics)")),
        ]
    };

    let list = List::new(items).block(
        Block::default()
            .title("🍄 LiveSpore Pipeline")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(list, area);
}

/// Render node status
fn render_node_status(frame: &mut Frame, area: Rect, standalone: bool) {
    let lines = if standalone {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "LiveSpore Integration",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from("LiveSpore is biomeOS's live"),
            Line::from("deployment system."),
            Line::from(""),
            Line::from("It manages:"),
            Line::from("  • Atomic deployments"),
            Line::from("  • Node coordination"),
            Line::from("  • Health monitoring"),
            Line::from("  • Rollback capabilities"),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Node Status",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Available Nodes: "),
                Span::styled("0", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::raw("Deployed: "),
                Span::styled("0", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::raw("Failed: "),
                Span::styled("0", Style::default().fg(Color::Red)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Deployment Status:",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from("  All systems ready"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Actions:",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from("  [Enter] View deployment"),
            Line::from("  [d] Deploy atomic"),
            Line::from("  [s] Stop deployment"),
            Line::from("  [r] Refresh"),
        ]
    };

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title("Node Management")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(paragraph, area);
}

