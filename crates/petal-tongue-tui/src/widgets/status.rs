// SPDX-License-Identifier: AGPL-3.0-only
//! Status Bar Widget

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
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
pub(crate) fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_zero_seconds() {
        assert_eq!(format_duration(std::time::Duration::from_secs(0)), "0s");
    }

    #[test]
    fn format_duration_seconds_only() {
        assert_eq!(format_duration(std::time::Duration::from_secs(45)), "45s");
    }

    #[test]
    fn format_duration_one_minute_thirty_seconds() {
        assert_eq!(
            format_duration(std::time::Duration::from_secs(90)),
            "1m 30s"
        );
    }

    #[test]
    fn format_duration_two_hours_fifteen_minutes() {
        assert_eq!(
            format_duration(std::time::Duration::from_secs(8100)),
            "2h 15m 0s"
        );
    }

    #[test]
    fn format_duration_hours_only() {
        assert_eq!(
            format_duration(std::time::Duration::from_secs(3600)),
            "1h 0m 0s"
        );
    }

    #[test]
    fn format_duration_mixed() {
        assert_eq!(
            format_duration(std::time::Duration::from_secs(3661)),
            "1h 1m 1s"
        );
    }
}
