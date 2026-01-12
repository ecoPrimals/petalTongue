//! Status Bar Widget

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::state::SystemStatus;

/// Status bar widget
pub struct StatusBar;

impl StatusBar {
    /// Render status bar
    pub fn render(frame: &mut Frame, area: Rect, status: &SystemStatus) {
        let status_text = vec![
            Span::raw("Primals: "),
            Span::styled(
                format!("{}", status.active_primals),
                Style::default().fg(if status.active_primals > 0 {
                    Color::Green
                } else {
                    Color::Yellow
                }),
            ),
            Span::raw(" | Devices: "),
            Span::styled(
                format!("{}", status.discovered_devices),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" | Uptime: "),
            Span::styled(
                format_duration(status.uptime),
                Style::default().fg(Color::Gray),
            ),
        ];

        let paragraph = Paragraph::new(Line::from(status_text));

        frame.render_widget(paragraph, area);
    }
}

/// Format duration as human-readable string
fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

