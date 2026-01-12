//! Logs View
//!
//! Real-time log streaming from all primals.
//! Leverages Songbird event stream if available.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::state::{LogLevel, TUIState};

/// Render logs view
pub fn render(frame: &mut Frame, area: Rect, state: &TUIState) {
    let logs = tokio::runtime::Handle::current().block_on(state.get_logs());
    let standalone = tokio::runtime::Handle::current().block_on(state.is_standalone());
    let selected = tokio::runtime::Handle::current().block_on(state.get_selected_index());

    // Split into log area and help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),      // Logs
            Constraint::Length(5),   // Help/controls
        ])
        .split(area);

    // Render logs
    render_log_list(frame, chunks[0], &logs, selected, standalone);

    // Render help
    render_help(frame, chunks[1]);
}

/// Render log list
fn render_log_list(
    frame: &mut Frame,
    area: Rect,
    logs: &[crate::state::LogMessage],
    selected: usize,
    standalone: bool,
) {
    let items: Vec<ListItem> = if logs.is_empty() {
        vec![
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                if standalone {
                    "⚠️  Standalone Mode - No live log streaming"
                } else {
                    "📜 No logs yet..."
                },
                Style::default().fg(Color::Gray),
            )])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::raw(
                "Logs will appear here as events occur.",
            )])),
        ]
    } else {
        logs.iter()
            .enumerate()
            .rev() // Show newest first
            .map(|(idx, log)| {
                let level_icon = match log.level {
                    LogLevel::Error => "❌",
                    LogLevel::Warn => "⚠️",
                    LogLevel::Info => "ℹ️",
                    LogLevel::Debug => "🐛",
                    LogLevel::Trace => "🔍",
                };

                let level_color = match log.level {
                    LogLevel::Error => Color::Red,
                    LogLevel::Warn => Color::Yellow,
                    LogLevel::Info => Color::Cyan,
                    LogLevel::Debug => Color::Magenta,
                    LogLevel::Trace => Color::DarkGray,
                };

                let timestamp = log.timestamp.format("%H:%M:%S");

                let source = log
                    .source
                    .as_ref()
                    .map(|s| format!("[{}]", s))
                    .unwrap_or_else(|| "[system]".to_string());

                let is_selected = idx == logs.len() - 1 - selected;

                let style = if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{} ", level_icon),
                        style.fg(level_color),
                    ),
                    Span::styled(
                        format!("[{}] ", timestamp),
                        style.fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("{} ", source),
                        style.fg(Color::Magenta),
                    ),
                    Span::styled(&log.message, style.fg(Color::White)),
                ]))
                .style(style)
            })
            .collect()
    };

    let title = format!("📜 Logs ({} total)", logs.len());

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(list, area);
}

/// Render help/controls
fn render_help(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(vec![
            Span::styled(
                "Controls:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("[↑/k ↓/j]", Style::default().fg(Color::Yellow)),
            Span::raw(" Navigate  "),
            Span::styled("[r]", Style::default().fg(Color::Yellow)),
            Span::raw(" Refresh  "),
            Span::styled("[Home/End]", Style::default().fg(Color::Yellow)),
            Span::raw(" Jump"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::raw("Logs are shown newest first. Scroll to see older entries."),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(Block::default().borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}

