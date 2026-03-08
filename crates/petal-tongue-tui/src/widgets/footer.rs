// SPDX-License-Identifier: AGPL-3.0-only
//! Footer Widget

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Footer widget
pub struct Footer;

impl Footer {
    /// Render footer with keyboard shortcuts
    pub fn render(frame: &mut Frame, area: Rect, standalone: bool) {
        let mode_indicator = if standalone {
            Span::styled("  [STANDALONE MODE]", Style::default().fg(Color::Yellow))
        } else {
            Span::raw("")
        };

        let shortcuts = vec![
            Span::raw("[1-8] Views | "),
            Span::raw("[↑/k ↓/j] Navigate | "),
            Span::raw("[r] Refresh | "),
            Span::raw("[?] Help | "),
            Span::styled("[q] Quit", Style::default().fg(Color::Red)),
            mode_indicator,
        ];

        let paragraph =
            Paragraph::new(Line::from(shortcuts)).block(Block::default().borders(Borders::ALL));

        frame.render_widget(paragraph, area);
    }
}
