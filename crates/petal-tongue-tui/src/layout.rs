// SPDX-License-Identifier: AGPL-3.0-only
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
