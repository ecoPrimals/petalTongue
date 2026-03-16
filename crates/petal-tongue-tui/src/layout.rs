// SPDX-License-Identifier: AGPL-3.0-or-later
//! Layout Management
//!
//! Manages responsive layouts for all views.
//! Pure Rust, zero hardcoding.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

/// Standard TUI layout with header, body, and footer
pub struct StandardLayout {
    /// Header area
    pub header: Rect,
    /// Body area
    pub body: Rect,
    /// Footer area
    pub footer: Rect,
}

impl StandardLayout {
    /// Create standard layout
    #[must_use]
    pub fn new(frame: &Frame) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Body (flexible)
                Constraint::Length(3), // Footer
            ])
            .split(frame.area());

        Self {
            header: chunks[0],
            body: chunks[1],
            footer: chunks[2],
        }
    }
}

/// Two-column layout (sidebar + main)
pub struct TwoColumnLayout {
    /// Left sidebar
    pub sidebar: Rect,
    /// Main content area
    pub main: Rect,
}

impl TwoColumnLayout {
    /// Create two-column layout
    #[must_use]
    pub fn new(area: Rect, sidebar_width: u16) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(sidebar_width), Constraint::Min(0)])
            .split(area);

        Self {
            sidebar: chunks[0],
            main: chunks[1],
        }
    }
}

/// Three-column layout
pub struct ThreeColumnLayout {
    /// Left column
    pub left: Rect,
    /// Center column
    pub center: Rect,
    /// Right column
    pub right: Rect,
}

impl ThreeColumnLayout {
    /// Create three-column layout
    #[must_use]
    pub fn new(area: Rect, left_width: u16, right_width: u16) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(left_width),
                Constraint::Min(0),
                Constraint::Length(right_width),
            ])
            .split(area);

        Self {
            left: chunks[0],
            center: chunks[1],
            right: chunks[2],
        }
    }
}

/// Split layout (top/bottom)
pub struct SplitLayout {
    /// Top area
    pub top: Rect,
    /// Bottom area
    pub bottom: Rect,
}

impl SplitLayout {
    /// Create split layout with percentage
    #[must_use]
    pub fn new(area: Rect, top_percent: u16) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(top_percent),
                Constraint::Percentage(100 - top_percent),
            ])
            .split(area);

        Self {
            top: chunks[0],
            bottom: chunks[1],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_column_layout_chunk_sizes() {
        let area = Rect::new(0, 0, 80, 24);
        let layout = TwoColumnLayout::new(area, 20);
        assert_eq!(layout.sidebar.width, 20);
        assert_eq!(layout.sidebar.height, 24);
        assert_eq!(layout.main.width, 60);
        assert_eq!(layout.main.height, 24);
    }

    #[test]
    fn three_column_layout_chunk_sizes() {
        let area = Rect::new(0, 0, 80, 24);
        let layout = ThreeColumnLayout::new(area, 15, 25);
        assert_eq!(layout.left.width, 15);
        assert_eq!(layout.center.width, 40);
        assert_eq!(layout.right.width, 25);
        assert_eq!(layout.left.height, 24);
    }

    #[test]
    fn split_layout_chunk_sizes() {
        let area = Rect::new(0, 0, 80, 24);
        let layout = SplitLayout::new(area, 50);
        assert_eq!(layout.top.height, 12);
        assert_eq!(layout.bottom.height, 12);
        assert_eq!(layout.top.width, 80);
    }

    #[test]
    fn split_layout_uneven_percentages() {
        let area = Rect::new(0, 0, 100, 20);
        let layout = SplitLayout::new(area, 30);
        assert_eq!(layout.top.height, 6);
        assert_eq!(layout.bottom.height, 14);
    }

    #[test]
    fn two_column_layout_minimal_area() {
        let area = Rect::new(0, 0, 10, 5);
        let layout = TwoColumnLayout::new(area, 3);
        assert_eq!(layout.sidebar.width, 3);
        assert_eq!(layout.main.width, 7);
        assert_eq!(layout.sidebar.x, 0);
        assert_eq!(layout.main.x, 3);
    }

    #[test]
    fn three_column_layout_small_center() {
        let area = Rect::new(0, 0, 50, 10);
        let layout = ThreeColumnLayout::new(area, 10, 15);
        assert_eq!(layout.left.width, 10);
        assert_eq!(layout.right.width, 15);
        assert_eq!(layout.center.width, 25);
    }

    #[test]
    fn standard_layout_requires_frame() {
        let backend = ratatui::backend::TestBackend::new(80, 24);
        let mut terminal = ratatui::Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                let layout = StandardLayout::new(frame);
                assert_eq!(layout.header.height, 3);
                assert_eq!(layout.body.height, 18);
                assert_eq!(layout.footer.height, 3);
                assert_eq!(layout.header.width, 80);
            })
            .expect("draw");
    }
}
