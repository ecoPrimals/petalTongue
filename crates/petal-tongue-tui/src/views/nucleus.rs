//! NUCLEUS View
//!
//! Secure discovery management for biomeOS.
//! Shows discovery layers and trust scores.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::state::TUIState;

/// Render NUCLEUS view
pub fn render(frame: &mut Frame, area: Rect, state: &TUIState) {
    let standalone = tokio::runtime::Handle::current().block_on(state.is_standalone());

    // Split into discovery layers and trust matrix
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Discovery layers
            Constraint::Percentage(50), // Trust matrix
        ])
        .split(area);

    // Render discovery layers
    render_discovery_layers(frame, chunks[0], standalone);

    // Render trust matrix
    render_trust_matrix(frame, chunks[1], standalone);
}

/// Render discovery layers
fn render_discovery_layers(frame: &mut Frame, area: Rect, standalone: bool) {
    let items: Vec<ListItem> = if standalone {
        vec![
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "⚠️  Standalone Mode",
                Style::default().fg(Color::Yellow),
            )])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from("NUCLEUS requires biomeOS.")),
        ]
    } else {
        vec![
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "🔐 Discovery Layers",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![
                Span::styled("Layer 1:", Style::default().fg(Color::Green)),
                Span::raw(" Local Filesystem"),
            ])),
            ListItem::new(Line::from("  Trust: High")),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![
                Span::styled("Layer 2:", Style::default().fg(Color::Cyan)),
                Span::raw(" Network (DNS-SD)"),
            ])),
            ListItem::new(Line::from("  Trust: Medium")),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![
                Span::styled("Layer 3:", Style::default().fg(Color::Yellow)),
                Span::raw(" External Discovery"),
            ])),
            ListItem::new(Line::from("  Trust: Low (verify)")),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "💡 Security:",
                Style::default().fg(Color::Cyan),
            )])),
            ListItem::new(Line::from("Each layer adds trust metadata")),
            ListItem::new(Line::from("for capability verification.")),
        ]
    };

    let list = List::new(items).block(
        Block::default()
            .title("🔐 NUCLEUS Discovery")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(list, area);
}

/// Render trust matrix
fn render_trust_matrix(frame: &mut Frame, area: Rect, standalone: bool) {
    let lines = if standalone {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "NUCLEUS Integration",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from("NUCLEUS is biomeOS's secure"),
            Line::from("discovery system."),
            Line::from(""),
            Line::from("It provides:"),
            Line::from("  • Multi-layer discovery"),
            Line::from("  • Trust scoring"),
            Line::from("  • Capability verification"),
            Line::from("  • Security policies"),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Trust Matrix",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Discovered Primals: "),
                Span::styled("0", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::raw("Trusted: "),
                Span::styled("0", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::raw("Unverified: "),
                Span::styled("0", Style::default().fg(Color::Red)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Security Policies:",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from("  ✅ Local filesystem"),
            Line::from("  ⚠️  Network discovery"),
            Line::from("  ❌ External sources"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Actions:",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from("  [Enter] View details"),
            Line::from("  [t] Trust primal"),
            Line::from("  [r] Refresh"),
        ]
    };

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title("Trust & Security")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(paragraph, area);
}
