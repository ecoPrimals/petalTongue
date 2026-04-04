// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC request dispatch: routes incoming methods to domain handlers on [`super::RpcHandlers`].

use super::RpcHandlers;
use super::{audio, graph, motor, system, ui, visualization, visualization_session};
use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse, error_codes};
use tracing::warn;

impl RpcHandlers {
    /// Dispatch JSON-RPC request to the appropriate handler
    pub async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let method = req.method.as_str();
        match method {
            // Health (with SEMANTIC_METHOD_NAMING_STANDARD aliases)
            "health.check" | "status" | "check" => system::handle_health_check(self, req),
            "health.liveness" | "ping" | "health" => system::handle_health_liveness(self, req),
            "health.readiness" => system::handle_health_readiness(self, req),
            "health.get" => system::get_health(self, req.id),

            // Identity + lifecycle (CAPABILITY_BASED_DISCOVERY / PRIMALSPRING)
            "identity.get" => system::handle_identity_get(self, req.id),
            "lifecycle.status" => system::handle_lifecycle_status(self, req.id),

            // Capabilities (canonical + aliases per SEMANTIC_METHOD_NAMING_STANDARD)
            "capabilities.list" | "capability.list" | "primal.capabilities" => {
                system::get_capabilities(self, req.id)
            }
            "capability.announce" => system::handle_announce_capabilities(self, req),
            "capabilities.sensory" => system::handle_capabilities_sensory(self, req),
            "capabilities.sensory.negotiate" => {
                system::handle_capabilities_sensory_negotiate(self, req)
            }

            // UI
            "ui.render" => ui::handle_ui_render(self, req).await,
            "ui.display_status" => ui::handle_ui_display_status(self, req),

            // Graph + topology
            "visualization.render.graph" => graph::render_graph(self, req.params, req.id).await,
            "topology.get" => system::get_topology(self, req.id),
            "visualization.render" => visualization::handle_render(self, req),
            "visualization.render.scene" => visualization::handle_render_scene(self, req),
            "visualization.session.list" => {
                visualization_session::handle_session_list(self, req.id)
            }
            "visualization.render.stream" => visualization::handle_stream(self, req),
            "visualization.render.grammar" => visualization::handle_grammar_render(self, req),
            "visualization.render.dashboard" => visualization::handle_dashboard_render(self, req),
            "visualization.validate" => visualization::handle_validate(self, req),
            "visualization.export" => visualization::handle_export(self, req),
            "visualization.dismiss" => visualization_session::handle_dismiss(self, req),
            "visualization.interact.apply" => visualization::handle_interact_apply(self, req),
            "visualization.interact.perspectives" => {
                visualization::handle_interact_perspectives(self, req.id)
            }
            "visualization.capabilities" => visualization::handle_capabilities(self, req.id),
            "visualization.session.status" => {
                visualization_session::handle_session_status(self, req)
            }
            "visualization.introspect" => visualization::handle_introspect(self, req.id),
            "visualization.panels" => visualization::handle_panels(self, req.id),
            "visualization.showing" => visualization::handle_showing(self, req),
            "interaction.subscribe" | "visualization.interact.subscribe" => {
                self.handle_interaction_subscribe(req)
            }
            "interaction.poll" | "visualization.interact.poll" => self.handle_interaction_poll(req),
            "interaction.unsubscribe" | "visualization.interact.unsubscribe" => {
                self.handle_interaction_unsubscribe(req)
            }
            "interaction.sensor_stream.subscribe" => self.handle_sensor_stream_subscribe(req),
            "interaction.sensor_stream.unsubscribe" => self.handle_sensor_stream_unsubscribe(req),
            "interaction.sensor_stream.poll" => self.handle_sensor_stream_poll(req),
            "provider.register_capability" => system::handle_provider_register(self, req),
            "audio.synthesize" => audio::handle_audio_synthesize(self, req),
            "motor.set_panel" | "motor.set_zoom" | "motor.fit_to_view" | "motor.set_mode"
            | "motor.navigate" => motor::handle_motor_command(self, req),
            _ => {
                warn!("Unknown method: {}", method);
                JsonRpcResponse::error(
                    req.id,
                    error_codes::METHOD_NOT_FOUND,
                    format!("Method not found: {method}"),
                )
            }
        }
    }
}
