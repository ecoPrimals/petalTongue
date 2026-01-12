//! TUI Views
//!
//! All 8 interactive views for the TUI.
//! Pure Rust, capability-based, graceful degradation.

mod dashboard;
mod topology;
mod logs;

use ratatui::Frame;
use tokio::runtime::Handle;

use crate::{
    layout::StandardLayout,
    state::TUIState,
    widgets::{Footer, Header},
};

/// Helper to block on async from sync context
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    Handle::current().block_on(future)
}

/// Render dashboard view
pub fn render_dashboard(frame: &mut Frame, state: &TUIState) {
    let layout = StandardLayout::new(frame);
    let view = block_on(state.get_view());
    let standalone = block_on(state.is_standalone());

    // Header
    Header::render(frame, layout.header, view);

    // Body - implemented dashboard
    dashboard::render(frame, layout.body, state);

    // Footer
    Footer::render(frame, layout.footer, standalone);
}

/// Render topology view
pub fn render_topology(frame: &mut Frame, state: &TUIState) {
    let layout = StandardLayout::new(frame);
    let view = block_on(state.get_view());
    let standalone = block_on(state.is_standalone());

    // Header
    Header::render(frame, layout.header, view);

    // Body - implemented topology
    topology::render(frame, layout.body, state);

    // Footer
    Footer::render(frame, layout.footer, standalone);
}

/// Render devices view
pub fn render_devices(frame: &mut Frame, state: &TUIState) {
    let layout = StandardLayout::new(frame);
    let view = block_on(state.get_view());
    let standalone = block_on(state.is_standalone());

    Header::render(frame, layout.header, view);
    render_placeholder(frame, layout.body, "Devices View - Coming Soon");
    Footer::render(frame, layout.footer, standalone);
}

/// Render primals view
pub fn render_primals(frame: &mut Frame, state: &TUIState) {
    let layout = StandardLayout::new(frame);
    let view = block_on(state.get_view());
    let standalone = block_on(state.is_standalone());

    Header::render(frame, layout.header, view);
    render_placeholder(frame, layout.body, "Primals View - Coming Soon");
    Footer::render(frame, layout.footer, standalone);
}

/// Render logs view
pub fn render_logs(frame: &mut Frame, state: &TUIState) {
    let layout = StandardLayout::new(frame);
    let view = block_on(state.get_view());
    let standalone = block_on(state.is_standalone());

    // Header
    Header::render(frame, layout.header, view);

    // Body - implemented logs
    logs::render(frame, layout.body, state);

    // Footer
    Footer::render(frame, layout.footer, standalone);
}

/// Render neuralAPI view
pub fn render_neural_api(frame: &mut Frame, state: &TUIState) {
    let layout = StandardLayout::new(frame);
    let view = block_on(state.get_view());
    let standalone = block_on(state.is_standalone());

    Header::render(frame, layout.header, view);
    render_placeholder(frame, layout.body, "neuralAPI View - Coming Soon");
    Footer::render(frame, layout.footer, standalone);
}

/// Render NUCLEUS view
pub fn render_nucleus(frame: &mut Frame, state: &TUIState) {
    let layout = StandardLayout::new(frame);
    let view = block_on(state.get_view());
    let standalone = block_on(state.is_standalone());

    Header::render(frame, layout.header, view);
    render_placeholder(frame, layout.body, "NUCLEUS View - Coming Soon");
    Footer::render(frame, layout.footer, standalone);
}

/// Render LiveSpore view
pub fn render_livespore(frame: &mut Frame, state: &TUIState) {
    let layout = StandardLayout::new(frame);
    let view = block_on(state.get_view());
    let standalone = block_on(state.is_standalone());

    Header::render(frame, layout.header, view);
    render_placeholder(frame, layout.body, "LiveSpore View - Coming Soon");
    Footer::render(frame, layout.footer, standalone);
}

/// Render placeholder content
fn render_placeholder(frame: &mut Frame, area: ratatui::layout::Rect, text: &str) {
    use ratatui::{
        style::{Color, Style},
        text::Line,
        widgets::{Block, Borders, Paragraph},
    };

    let paragraph = Paragraph::new(Line::from(text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(paragraph, area);
}

