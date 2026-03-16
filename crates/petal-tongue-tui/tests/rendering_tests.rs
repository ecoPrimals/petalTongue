// SPDX-License-Identifier: AGPL-3.0-only
#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::Utc;
use petal_tongue_core::{PrimalHealthStatus, PrimalId, PrimalInfo};
use petal_tongue_tui::state::{LogLevel, LogMessage, TUIState, View};
use ratatui::{Terminal, backend::TestBackend};

mod common;
use common::{create_test_edge, create_test_primal};

fn buffer_text(buffer: &ratatui::buffer::Buffer) -> String {
    buffer
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect()
}

async fn render_and_get_buffer<F>(state: &TUIState, render_fn: F) -> ratatui::buffer::Buffer
where
    F: FnOnce(&mut ratatui::Frame, &TUIState) + Send + 'static,
{
    let state = state.clone();
    tokio::task::spawn_blocking(move || {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render_fn(frame, &state)).unwrap();
        terminal.backend().buffer().clone()
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn test_dashboard_rendering() {
    let state = TUIState::new();
    let primals = vec![
        PrimalInfo::new(
            PrimalId::from("songbird-1"),
            "songbird",
            "Discovery",
            "http://localhost",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ),
        PrimalInfo::new(
            PrimalId::from("toadstool-1"),
            "toadstool",
            "Compute",
            "http://localhost",
            vec![],
            PrimalHealthStatus::Warning,
            0,
        ),
    ];
    state.update_primals(primals).await;

    let buffer = render_and_get_buffer(&state, petal_tongue_tui::views::render_dashboard).await;
    let text = buffer_text(&buffer);
    assert!(text.contains("Dashboard"));
    assert!(text.contains("petalTongue Dashboard"));
    assert!(text.contains("songbird"));
    assert!(text.contains("toadstool"));
    assert!(text.contains("Healthy"));
    assert!(text.contains("Warning"));
    assert!(text.contains("Primals"));
    assert!(text.contains("Topology"));
}

#[tokio::test]
async fn test_primals_view_rendering() {
    let state = TUIState::new();
    state.set_view(View::Primals).await;
    let primals = vec![
        create_test_primal("songbird", "songbird-1"),
        create_test_primal("toadstool", "toadstool-1"),
    ];
    state.update_primals(primals).await;

    let buffer = render_and_get_buffer(&state, petal_tongue_tui::views::render_primals).await;
    let text = buffer_text(&buffer);
    assert!(text.contains("Primals"));
    assert!(text.contains("songbird"));
    assert!(text.contains("toadstool"));
    assert!(text.contains("Primal Details"));
}

#[tokio::test]
async fn test_logs_view_rendering() {
    let state = TUIState::new();
    state.set_view(View::Logs).await;
    state
        .add_log(LogMessage {
            timestamp: Utc::now(),
            source: Some("songbird".to_string()),
            level: LogLevel::Info,
            message: "Discovery event 1".to_string(),
        })
        .await;
    state
        .add_log(LogMessage {
            timestamp: Utc::now(),
            source: Some("toadstool".to_string()),
            level: LogLevel::Error,
            message: "Compute error".to_string(),
        })
        .await;

    let buffer = render_and_get_buffer(&state, petal_tongue_tui::views::render_logs).await;
    let text = buffer_text(&buffer);
    assert!(text.contains("Logs"));
    assert!(text.contains("Discovery event 1"));
    assert!(text.contains("Compute error"));
    assert!(text.contains("songbird"));
    assert!(text.contains("toadstool"));
}

#[tokio::test]
async fn test_topology_view_rendering() {
    let state = TUIState::new();
    state.set_view(View::Topology).await;
    let primals = vec![
        create_test_primal("songbird", "songbird-1"),
        create_test_primal("toadstool", "toadstool-1"),
    ];
    state.update_primals(primals).await;
    let topology = vec![create_test_edge("songbird-1", "toadstool-1", "discovery")];
    state.update_topology(topology).await;

    let buffer = render_and_get_buffer(&state, petal_tongue_tui::views::render_topology).await;
    let text = buffer_text(&buffer);
    assert!(text.contains("Topology"));
    assert!(text.contains("songbird"));
    assert!(text.contains("toadstool"));
    assert!(text.contains("discovery"));
    assert!(text.contains("Topology Graph"));
    assert!(text.contains("Details"));
}

#[tokio::test]
async fn test_devices_view_rendering_zero_devices() {
    let state = TUIState::new();
    state.set_view(View::Devices).await;

    let buffer = render_and_get_buffer(&state, petal_tongue_tui::views::render_devices).await;
    let text = buffer_text(&buffer);
    assert!(text.contains("Devices"));
    assert!(text.contains("No devices discovered yet"));
}

#[tokio::test]
async fn test_header_footer_widgets() {
    let state = TUIState::new();
    state.set_view(View::Dashboard).await;

    let buffer = render_and_get_buffer(&state, petal_tongue_tui::views::render_dashboard).await;
    let text = buffer_text(&buffer);
    assert!(text.contains("Dashboard"));
    assert!(text.contains("petalTongue"));
    assert!(text.contains("[1-8]"));
    assert!(text.contains("[q] Quit"));
}

#[tokio::test]
async fn test_nucleus_view_rendering() {
    let state = TUIState::new();
    state.set_view(View::Nucleus).await;

    let buffer = render_and_get_buffer(&state, petal_tongue_tui::views::render_nucleus).await;
    let text = buffer_text(&buffer);
    assert!(text.contains("NUCLEUS"));
    assert!(text.contains("NUCLEUS Discovery"));
    assert!(text.contains("Trust & Security"));
    assert!(text.contains("Discovery Layers"));
    assert!(text.contains("Trust Matrix"));
}

#[tokio::test]
async fn test_livespore_view_rendering() {
    let state = TUIState::new();
    state.set_view(View::LiveSpore).await;

    let buffer = render_and_get_buffer(&state, petal_tongue_tui::views::render_livespore).await;
    let text = buffer_text(&buffer);
    assert!(text.contains("LiveSpore"));
    assert!(text.contains("LiveSpore Pipeline"));
    assert!(text.contains("Node Management"));
    assert!(text.contains("Deployment Pipeline"));
    assert!(text.contains("Node Status"));
}

#[tokio::test]
async fn test_neural_api_view_rendering() {
    let state = TUIState::new();
    state.set_view(View::NeuralAPI).await;

    let buffer = render_and_get_buffer(&state, petal_tongue_tui::views::render_neural_api).await;
    let text = buffer_text(&buffer);
    assert!(text.contains("neuralAPI"));
    assert!(text.contains("neuralAPI Graphs"));
    assert!(text.contains("Execution Details"));
    assert!(text.contains("Neural Graphs"));
    assert!(text.contains("Execution Status"));
}

#[tokio::test]
async fn test_standard_layout_chunk_sizes() {
    use petal_tongue_tui::layout::StandardLayout;

    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut layout_result = None;
    terminal
        .draw(|frame| {
            let layout = StandardLayout::new(frame);
            layout_result = Some((layout.header, layout.body, layout.footer));
        })
        .unwrap();

    let (header, body, footer) = layout_result.unwrap();
    assert_eq!(header.width, 120);
    assert_eq!(header.height, 3);
    assert_eq!(body.width, 120);
    assert_eq!(body.height, 34);
    assert_eq!(footer.width, 120);
    assert_eq!(footer.height, 3);
}
