// SPDX-License-Identifier: AGPL-3.0-or-later
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

/// Render `LiveSpore` view
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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[tokio::test]
    async fn test_render_dashboard_no_panic() {
        let state = TUIState::new();
        tokio::task::spawn_blocking(move || {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).expect("terminal");
            terminal
                .draw(|frame| render_dashboard(frame, &state))
                .expect("render");
        })
        .await
        .expect("spawn");
    }

    #[tokio::test]
    async fn test_render_topology_no_panic() {
        let state = TUIState::new();
        tokio::task::spawn_blocking(move || {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).expect("terminal");
            terminal
                .draw(|frame| render_topology(frame, &state))
                .expect("render");
        })
        .await
        .expect("spawn");
    }

    #[tokio::test]
    async fn test_render_topology_standalone() {
        let state = TUIState::new();
        state.set_standalone_mode(true).await;
        tokio::task::spawn_blocking(move || {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).expect("terminal");
            terminal
                .draw(|frame| render_topology(frame, &state))
                .expect("render");
        })
        .await
        .expect("spawn");
    }

    #[tokio::test]
    async fn test_render_dashboard_standalone() {
        let state = TUIState::new();
        state.set_standalone_mode(true).await;
        tokio::task::spawn_blocking(move || {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).expect("terminal");
            terminal
                .draw(|frame| render_dashboard(frame, &state))
                .expect("render");
        })
        .await
        .expect("spawn");
    }

    #[tokio::test]
    async fn test_render_devices_standalone() {
        let state = TUIState::new();
        state.set_standalone_mode(true).await;
        tokio::task::spawn_blocking(move || {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).expect("terminal");
            terminal
                .draw(|frame| render_devices(frame, &state))
                .expect("render");
        })
        .await
        .expect("spawn");
    }

    #[tokio::test]
    async fn test_render_primals_standalone() {
        let state = TUIState::new();
        state.set_standalone_mode(true).await;
        tokio::task::spawn_blocking(move || {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).expect("terminal");
            terminal
                .draw(|frame| render_primals(frame, &state))
                .expect("render");
        })
        .await
        .expect("spawn");
    }

    #[tokio::test]
    async fn test_render_dashboard_with_primals() {
        let state = TUIState::new();
        state.set_standalone_mode(false).await;
        state
            .update_primals(vec![petal_tongue_core::PrimalInfo::new(
                "p1",
                "TestPrimal",
                "Compute",
                "unix:///tmp/p1.sock",
                vec![],
                petal_tongue_core::PrimalHealthStatus::Healthy,
                0,
            )])
            .await;
        tokio::task::spawn_blocking(move || {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).expect("terminal");
            terminal
                .draw(|frame| render_dashboard(frame, &state))
                .expect("render");
        })
        .await
        .expect("spawn");
    }

    #[tokio::test]
    async fn test_render_topology_with_primals() {
        let state = TUIState::new();
        state.set_standalone_mode(false).await;
        state
            .update_primals(vec![petal_tongue_core::PrimalInfo::new(
                "p1",
                "A",
                "T",
                "unix:///tmp/p1.sock",
                vec![],
                petal_tongue_core::PrimalHealthStatus::Healthy,
                0,
            )])
            .await;
        tokio::task::spawn_blocking(move || {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).expect("terminal");
            terminal
                .draw(|frame| render_topology(frame, &state))
                .expect("render");
        })
        .await
        .expect("spawn");
    }

    #[tokio::test]
    async fn test_render_topology_with_topology_edges() {
        let state = TUIState::new();
        state.set_standalone_mode(false).await;
        state
            .update_primals(vec![
                petal_tongue_core::PrimalInfo::new(
                    "a",
                    "A",
                    "T",
                    "unix:///tmp/a.sock",
                    vec![],
                    petal_tongue_core::PrimalHealthStatus::Healthy,
                    0,
                ),
                petal_tongue_core::PrimalInfo::new(
                    "b",
                    "B",
                    "T",
                    "unix:///tmp/b.sock",
                    vec![],
                    petal_tongue_core::PrimalHealthStatus::Healthy,
                    0,
                ),
            ])
            .await;
        state
            .update_topology(vec![petal_tongue_core::TopologyEdge {
                from: petal_tongue_core::PrimalId::from("a"),
                to: petal_tongue_core::PrimalId::from("b"),
                edge_type: "conn".to_string(),
                label: None,
                capability: None,
                metrics: None,
            }])
            .await;
        tokio::task::spawn_blocking(move || {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).expect("terminal");
            terminal
                .draw(|frame| render_topology(frame, &state))
                .expect("render");
        })
        .await
        .expect("spawn");
    }

    #[tokio::test]
    async fn test_render_primals_with_data() {
        let state = TUIState::new();
        state.set_standalone_mode(false).await;
        state
            .update_primals(vec![petal_tongue_core::PrimalInfo::new(
                "p1",
                "Primal1",
                "Compute",
                "unix:///tmp/p1.sock",
                vec![],
                petal_tongue_core::PrimalHealthStatus::Healthy,
                0,
            )])
            .await;
        state.set_selected_index(0).await;
        tokio::task::spawn_blocking(move || {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).expect("terminal");
            terminal
                .draw(|frame| render_primals(frame, &state))
                .expect("render");
        })
        .await
        .expect("spawn");
    }

    #[tokio::test]
    async fn test_render_devices_with_count() {
        let state = TUIState::new();
        state.set_standalone_mode(false).await;
        tokio::task::spawn_blocking(move || {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).expect("terminal");
            terminal
                .draw(|frame| render_devices(frame, &state))
                .expect("render");
        })
        .await
        .expect("spawn");
    }

    #[tokio::test]
    async fn test_render_dashboard_with_topology() {
        let state = TUIState::new();
        state.set_standalone_mode(false).await;
        state
            .update_primals(vec![
                petal_tongue_core::PrimalInfo::new(
                    "a",
                    "A",
                    "T",
                    "unix:///tmp/a.sock",
                    vec![],
                    petal_tongue_core::PrimalHealthStatus::Healthy,
                    0,
                ),
                petal_tongue_core::PrimalInfo::new(
                    "b",
                    "B",
                    "T",
                    "unix:///tmp/b.sock",
                    vec![],
                    petal_tongue_core::PrimalHealthStatus::Healthy,
                    0,
                ),
            ])
            .await;
        state
            .update_topology(vec![
                petal_tongue_core::TopologyEdge {
                    from: petal_tongue_core::PrimalId::from("a"),
                    to: petal_tongue_core::PrimalId::from("b"),
                    edge_type: "e1".to_string(),
                    label: None,
                    capability: None,
                    metrics: None,
                },
                petal_tongue_core::TopologyEdge {
                    from: petal_tongue_core::PrimalId::from("b"),
                    to: petal_tongue_core::PrimalId::from("a"),
                    edge_type: "e2".to_string(),
                    label: None,
                    capability: None,
                    metrics: None,
                },
                petal_tongue_core::TopologyEdge {
                    from: petal_tongue_core::PrimalId::from("a"),
                    to: petal_tongue_core::PrimalId::from("b"),
                    edge_type: "e3".to_string(),
                    label: None,
                    capability: None,
                    metrics: None,
                },
                petal_tongue_core::TopologyEdge {
                    from: petal_tongue_core::PrimalId::from("b"),
                    to: petal_tongue_core::PrimalId::from("a"),
                    edge_type: "e4".to_string(),
                    label: None,
                    capability: None,
                    metrics: None,
                },
                petal_tongue_core::TopologyEdge {
                    from: petal_tongue_core::PrimalId::from("a"),
                    to: petal_tongue_core::PrimalId::from("b"),
                    edge_type: "e5".to_string(),
                    label: None,
                    capability: None,
                    metrics: None,
                },
                petal_tongue_core::TopologyEdge {
                    from: petal_tongue_core::PrimalId::from("b"),
                    to: petal_tongue_core::PrimalId::from("a"),
                    edge_type: "e6".to_string(),
                    label: None,
                    capability: None,
                    metrics: None,
                },
            ])
            .await;
        tokio::task::spawn_blocking(move || {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).expect("terminal");
            terminal
                .draw(|frame| render_dashboard(frame, &state))
                .expect("render");
        })
        .await
        .expect("spawn");
    }

    #[tokio::test]
    async fn test_render_devices_no_panic() {
        let state = TUIState::new();
        tokio::task::spawn_blocking(move || {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).expect("terminal");
            terminal
                .draw(|frame| render_devices(frame, &state))
                .expect("render");
        })
        .await
        .expect("spawn");
    }

    #[tokio::test]
    async fn test_render_primals_no_panic() {
        let state = TUIState::new();
        tokio::task::spawn_blocking(move || {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).expect("terminal");
            terminal
                .draw(|frame| render_primals(frame, &state))
                .expect("render");
        })
        .await
        .expect("spawn");
    }

    #[tokio::test]
    async fn test_render_logs_no_panic() {
        let state = TUIState::new();
        tokio::task::spawn_blocking(move || {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).expect("terminal");
            terminal
                .draw(|frame| render_logs(frame, &state))
                .expect("render");
        })
        .await
        .expect("spawn");
    }

    #[tokio::test]
    async fn test_render_neural_api_no_panic() {
        let state = TUIState::new();
        tokio::task::spawn_blocking(move || {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).expect("terminal");
            terminal
                .draw(|frame| render_neural_api(frame, &state))
                .expect("render");
        })
        .await
        .expect("spawn");
    }

    #[tokio::test]
    async fn test_render_nucleus_no_panic() {
        let state = TUIState::new();
        tokio::task::spawn_blocking(move || {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).expect("terminal");
            terminal
                .draw(|frame| render_nucleus(frame, &state))
                .expect("render");
        })
        .await
        .expect("spawn");
    }

    #[tokio::test]
    async fn test_render_livespore_no_panic() {
        let state = TUIState::new();
        tokio::task::spawn_blocking(move || {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).expect("terminal");
            terminal
                .draw(|frame| render_livespore(frame, &state))
                .expect("render");
        })
        .await
        .expect("spawn");
    }
}
