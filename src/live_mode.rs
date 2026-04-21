// SPDX-License-Identifier: AGPL-3.0-or-later
//! Live mode - NUCLEUS interactive desktop
//!
//! Merges `ui` (egui/eframe native window) and `server` (UDS JSON-RPC IPC)
//! into a single process. The IPC server runs as a background tokio task,
//! the egui window runs on the main thread, connected via shared state
//! (`Arc<RwLock<VisualizationState>>` and companion registries).
//!
//! This is the tier-one deployment mode for interactive desktop NUCLEUS:
//! domain logic pushes scene data via `visualization.render.scene`, the egui
//! window renders it, user input flows back via `interaction.poll`.

use crate::data_service::DataService;
use crate::error::AppError;
use petal_tongue_core::constants::PRIMAL_NAME;
use petal_tongue_ipc::UnixSocketServer;
use std::sync::Arc;

type Result<T> = std::result::Result<T, AppError>;

/// Run live mode: IPC server (background) + egui window (main thread).
pub async fn run(
    scenario: Option<String>,
    no_audio: bool,
    data_service: Arc<DataService>,
    tcp_port: Option<u16>,
    socket_path: Option<String>,
) -> Result<()> {
    let graph = data_service.graph();

    let (motor_tx, motor_rx) = std::sync::mpsc::channel();

    let socket_override = socket_path.map(std::path::PathBuf::from);
    let mut server = UnixSocketServer::new_with_socket(graph.clone(), socket_override)
        .map_err(|e| AppError::Other(format!("Failed to create IPC server: {e}")))?
        .with_motor_sender(motor_tx);

    if let Some(port) = tcp_port {
        server = server.with_tcp_port(port);
    }

    let viz_state = server.visualization_state_handle();
    let sensor_stream = server.sensor_stream_handle();
    let interaction_subs = server.interaction_subscribers_handle();
    let callback_tx = server.callback_sender();

    let server = Arc::new(server);

    // Background: IPC accept loop
    let server_handle = {
        let server = Arc::clone(&server);
        tokio::spawn(async move {
            if let Err(e) = server.start().await {
                tracing::error!("IPC server error in live mode: {e}");
            }
        })
    };

    // Background: motor drain (UI does not consume motor_rx directly today)
    tokio::task::spawn_blocking(move || {
        while let Ok(cmd) = motor_rx.recv() {
            tracing::debug!(?cmd, "motor command received (live mode)");
        }
    });

    // Background: periodic capability discovery refresh
    let refresh_service = Arc::clone(&data_service);
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(petal_tongue_core::constants::default_heartbeat_interval());
        loop {
            interval.tick().await;
            if let Err(e) = refresh_service.refresh().await {
                tracing::warn!("periodic discovery refresh failed: {e}");
            }
        }
    });

    tracing::info!("🔌 Live mode starting (IPC server + native GUI)");

    // Main thread: egui window (blocks until window is closed)
    tokio::task::spawn_blocking(move || {
        run_ui_blocking(scenario, no_audio, &graph, viz_state, sensor_stream, interaction_subs, callback_tx)
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))??;

    server_handle.abort();
    Ok(())
}

fn run_ui_blocking(
    scenario: Option<String>,
    _no_audio: bool,
    graph: &Arc<std::sync::RwLock<petal_tongue_core::GraphEngine>>,
    viz_state: Arc<std::sync::RwLock<petal_tongue_ipc::VisualizationState>>,
    sensor_stream: Arc<std::sync::RwLock<petal_tongue_ipc::SensorStreamRegistry>>,
    interaction_subs: Arc<std::sync::RwLock<petal_tongue_ipc::InteractionSubscriberRegistry>>,
    callback_tx: Option<tokio::sync::mpsc::UnboundedSender<petal_tongue_ipc::CallbackDispatch>>,
) -> Result<()> {
    use petal_tongue_core::{InstanceId, RenderingCapabilities};
    use petal_tongue_ui::PetalTongueApp;

    let instance_id = InstanceId::new();
    tracing::info!(
        "🌸 Starting {} live instance: {}",
        PRIMAL_NAME,
        instance_id.as_str()
    );

    let scenario_path = crate::ui_mode::scenario_to_path(scenario);
    let capabilities = RenderingCapabilities::detect();

    let options = petal_tongue_ui::eframe::NativeOptions {
        viewport: petal_tongue_ui::egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title(crate::ui_mode::window_title())
            .with_visible(true)
            .with_active(true),
        ..Default::default()
    };

    let shared_graph = Arc::clone(graph);

    petal_tongue_ui::eframe::run_native(
        PRIMAL_NAME,
        options,
        Box::new(move |cc| {
            let mut app = PetalTongueApp::new_with_shared_graph(
                cc,
                scenario_path,
                capabilities,
                shared_graph,
            )?;
            app.set_visualization_state(viz_state);
            app.set_sensor_stream(sensor_stream);
            app.set_interaction_subscribers(interaction_subs);
            if let Some(tx) = callback_tx {
                app.set_callback_tx(tx);
            }
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| AppError::Eframe(e.to_string()))
}
