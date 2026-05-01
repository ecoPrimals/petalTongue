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

/// Create the IPC server with shared state handles for live mode.
///
/// Returns the server and motor channel endpoints. Separated from
/// [`run_on_main_thread`] so the server setup can be tested without a display.
fn create_live_server(
    data_service: &Arc<DataService>,
    tcp_port: Option<u16>,
    socket_path: Option<String>,
) -> Result<(
    UnixSocketServer,
    std::sync::mpsc::Sender<petal_tongue_core::MotorCommand>,
    std::sync::mpsc::Receiver<petal_tongue_core::MotorCommand>,
)> {
    let graph = data_service.graph();
    let (motor_tx, motor_rx) = std::sync::mpsc::channel();

    let socket_override = socket_path.map(std::path::PathBuf::from);
    let rendering_awareness = Arc::new(std::sync::RwLock::new(
        petal_tongue_core::RenderingAwareness::new(),
    ));
    let mut server = UnixSocketServer::new_with_socket(graph, socket_override)
        .map_err(|e| AppError::Other(format!("Failed to create IPC server: {e}")))?
        .with_rendering_awareness(Arc::clone(&rendering_awareness))
        .with_motor_sender(motor_tx.clone());

    if let Some(port) = tcp_port {
        server = server.with_tcp_port(port);
    }

    Ok((server, motor_tx, motor_rx))
}

/// Run live mode: IPC server (background tokio tasks) + egui window (main thread).
///
/// winit requires the event loop on the main thread (Linux X11/Wayland).
/// Background tasks (IPC server, motor drain, discovery refresh) are spawned
/// on the provided tokio `runtime`. The eframe event loop runs directly on
/// the calling (main) thread and blocks until the window is closed.
pub fn run_on_main_thread(
    scenario: Option<String>,
    _no_audio: bool,
    data_service: &Arc<DataService>,
    tcp_port: Option<u16>,
    socket_path: Option<String>,
    runtime: &tokio::runtime::Runtime,
) -> Result<()> {
    use petal_tongue_core::{InstanceId, RenderingCapabilities};
    use petal_tongue_ui::PetalTongueApp;

    let (server, motor_tx, motor_rx) = create_live_server(data_service, tcp_port, socket_path)?;

    let viz_state = server.visualization_state_handle();
    let sensor_stream = server.sensor_stream_handle();
    let interaction_subs = server.interaction_subscribers_handle();
    let callback_tx = server.callback_sender();

    let server = Arc::new(server);

    let ipc_server = Arc::clone(&server);
    runtime.spawn(async move {
        if let Err(e) = ipc_server.start().await {
            tracing::error!("IPC server error in live mode: {e}");
        }
    });

    let refresh_service = Arc::clone(data_service);
    runtime.spawn(async move {
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

    let instance_id = InstanceId::new();
    tracing::info!(
        "🌸 Starting {} live instance: {}",
        PRIMAL_NAME,
        instance_id.as_str()
    );

    let scenario_path = crate::ui_mode::scenario_to_path(scenario);
    let capabilities = RenderingCapabilities::detect();

    let viewport = petal_tongue_ui::egui::ViewportBuilder::default()
        .with_inner_size([1400.0, 900.0])
        .with_min_inner_size([800.0, 600.0])
        .with_title(crate::ui_mode::window_title())
        .with_visible(true)
        .with_active(true);
    let options = crate::ui_mode::native_options_with_any_thread(viewport);

    let shared_graph = data_service.graph();

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
            app.replace_motor_channel(motor_tx, motor_rx);
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| AppError::Eframe(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_data_service() -> Arc<DataService> {
        Arc::new(DataService::new())
    }

    #[test]
    fn create_live_server_succeeds_with_defaults() {
        let ds = test_data_service();
        let result = create_live_server(&ds, None, None);
        assert!(result.is_ok(), "server creation should succeed standalone");
    }

    #[test]
    fn create_live_server_with_tcp_port() {
        let ds = test_data_service();
        let result = create_live_server(&ds, Some(0), None);
        assert!(
            result.is_ok(),
            "server creation with TCP port 0 should succeed"
        );
    }

    #[test]
    fn create_live_server_with_socket_override() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let sock = tmp.path().join("live-test.sock");
        let ds = test_data_service();
        let result = create_live_server(&ds, None, Some(sock.to_string_lossy().into_owned()));
        assert!(
            result.is_ok(),
            "server creation with socket override should succeed"
        );
    }

    #[test]
    fn server_has_shared_state_handles() {
        let ds = test_data_service();
        let (server, _tx, _rx) = create_live_server(&ds, None, None).expect("server");
        let _viz = server.visualization_state_handle();
        let _sensor = server.sensor_stream_handle();
        let _subs = server.interaction_subscribers_handle();
    }

    #[test]
    fn motor_channel_is_functional() {
        use petal_tongue_core::MotorCommand;

        let ds = test_data_service();
        let (_server, tx, rx) = create_live_server(&ds, None, None).expect("server");
        tx.send(MotorCommand::RenderFrame { frame_id: 42 })
            .expect("send ok");
        let msg = rx.recv().expect("recv ok");
        assert!(
            matches!(msg, MotorCommand::RenderFrame { frame_id: 42 }),
            "expected RenderFrame command"
        );
    }
}
