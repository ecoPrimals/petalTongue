//! Header Widget

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::state::View;

/// Header widget
pub struct Header;

impl Header {
    /// Render header
    pub fn render(frame: &mut Frame, area: Rect, current_view: View) {
        let title = vec![
            Span::raw("🌸 "),
            Span::styled(
                "petalTongue",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Universal Interface (TUI Mode) - "),
            Span::styled(
                current_view.name(),
                Style::default().fg(Color::Yellow),
            ),
        ];

        let paragraph = Paragraph::new(Line::from(title))
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(paragraph, area);
    }
}

