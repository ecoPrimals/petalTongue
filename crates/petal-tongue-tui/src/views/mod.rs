//! TUI Views
//!
//! All 8 interactive views for the TUI.
//! Pure Rust, capability-based, graceful degradation.

mod dashboard;
mod devices;
mod livespore;
mod logs;
mod neural_api;
mod nucleus;
mod primals;
mod topology;

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

    // Header
    Header::render(frame, layout.header, view);

    // Body - implemented devices
    devices::render(frame, layout.body, state);

    // Footer
    Footer::render(frame, layout.footer, standalone);
}

/// Render primals view
pub fn render_primals(frame: &mut Frame, state: &TUIState) {
    let layout = StandardLayout::new(frame);
    let view = block_on(state.get_view());
    let standalone = block_on(state.is_standalone());

    // Header
    Header::render(frame, layout.header, view);

    // Body - implemented primals
    primals::render(frame, layout.body, state);

    // Footer
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

    // Header
    Header::render(frame, layout.header, view);

    // Body - implemented neuralAPI
    neural_api::render(frame, layout.body, state);

    // Footer
    Footer::render(frame, layout.footer, standalone);
}

/// Render NUCLEUS view
pub fn render_nucleus(frame: &mut Frame, state: &TUIState) {
    let layout = StandardLayout::new(frame);
    let view = block_on(state.get_view());
    let standalone = block_on(state.is_standalone());

    // Header
    Header::render(frame, layout.header, view);

    // Body - implemented NUCLEUS
    nucleus::render(frame, layout.body, state);

    // Footer
    Footer::render(frame, layout.footer, standalone);
}

/// Render LiveSpore view
pub fn render_livespore(frame: &mut Frame, state: &TUIState) {
    let layout = StandardLayout::new(frame);
    let view = block_on(state.get_view());
    let standalone = block_on(state.is_standalone());

    // Header
    Header::render(frame, layout.header, view);

    // Body - implemented LiveSpore
    livespore::render(frame, layout.body, state);

    // Footer
    Footer::render(frame, layout.footer, standalone);
}
