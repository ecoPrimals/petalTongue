//! neuralAPI View
//!
//! Graph orchestration management for biomeOS.
//! Shows neural graph execution and status.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::state::TUIState;

/// Render neuralAPI view
pub fn render(frame: &mut Frame, area: Rect, state: &TUIState) {
    let standalone = tokio::runtime::Handle::current().block_on(state.is_standalone());

    // Split into graph list and execution details
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Graph list
            Constraint::Percentage(50), // Execution details
        ])
        .split(area);

    // Render graph list
    render_graph_list(frame, chunks[0], standalone);

    // Render execution details
    render_execution_details(frame, chunks[1], standalone);
}

/// Render graph list
fn render_graph_list(frame: &mut Frame, area: Rect, standalone: bool) {
    let items: Vec<ListItem> = if standalone {
        vec![
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "⚠️  Standalone Mode",
                Style::default().fg(Color::Yellow),
            )])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from("neuralAPI requires biomeOS.")),
        ]
    } else {
        vec![
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "🧬 Neural Graphs",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "🔍 Discovering graphs...",
                Style::default().fg(Color::Gray),
            )])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from("Graphs will appear here once")),
            ListItem::new(Line::from("defined in biomeOS.")),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "💡 Example Graphs:",
                Style::default().fg(Color::Cyan),
            )])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from("  • Device Management")),
            ListItem::new(Line::from("  • Primal Orchestration")),
            ListItem::new(Line::from("  • Topology Discovery")),
            ListItem::new(Line::from("  • Health Monitoring")),
        ]
    };

    let list = List::new(items).block(
        Block::default()
            .title("🧬 neuralAPI Graphs")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(list, area);
}

/// Render execution details
fn render_execution_details(frame: &mut Frame, area: Rect, standalone: bool) {
    let lines = if standalone {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "neuralAPI Integration",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from("neuralAPI is biomeOS's graph"),
            Line::from("orchestration system."),
            Line::from(""),
            Line::from("It manages:"),
            Line::from("  • Graph definitions"),
            Line::from("  • Node execution"),
            Line::from("  • Data flow"),
            Line::from("  • Error handling"),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Execution Status",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Active Graphs: "),
                Span::styled("0", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::raw("Completed: "),
                Span::styled("0", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::raw("Failed: "),
                Span::styled("0", Style::default().fg(Color::Red)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Actions:",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from("  [Enter] View graph"),
            Line::from("  [e] Execute graph"),
            Line::from("  [s] Stop execution"),
            Line::from("  [r] Refresh"),
        ]
    };

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title("Execution Details")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(paragraph, area);
}
