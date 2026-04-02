// SPDX-License-Identifier: AGPL-3.0-or-later
//! Centralized capability string constants for IPC discovery.
//!
//! These are **capability identifiers**, not primal names. They describe
//! what a service can do, enabling capability-based discovery without
//! hardcoding knowledge of specific primals.
//!
//! # Convention
//!
//! All capability strings follow `{domain}.{operation}[.{variant}]` semantic
//! naming per wateringHole `SEMANTIC_METHOD_NAMING_STANDARD.md`.

/// Capabilities that petalTongue announces to the ecosystem.
#[expect(missing_docs, reason = "constant names mirror their semantic values")]
pub mod self_capabilities {
    pub const UI_RENDER: &str = "ui.render";
    pub const UI_VISUALIZATION: &str = "ui.visualization";
    pub const UI_GRAPH: &str = "ui.graph";
    pub const UI_TERMINAL: &str = "ui.terminal";
    pub const UI_AUDIO: &str = "ui.audio";

    pub const VISUALIZATION_RENDER: &str = "visualization.render";
    pub const VISUALIZATION_RENDER_STREAM: &str = "visualization.render.stream";
    pub const VISUALIZATION_RENDER_GRAMMAR: &str = "visualization.render.grammar";
    pub const VISUALIZATION_RENDER_DASHBOARD: &str = "visualization.render.dashboard";
    pub const VISUALIZATION_INTERACT: &str = "visualization.interact";
    pub const VISUALIZATION_INTERACT_SUBSCRIBE: &str = "visualization.interact.subscribe";
    pub const VISUALIZATION_PROVENANCE: &str = "visualization.provenance";
    pub const VISUALIZATION_RENDER_SCENE: &str = "visualization.render.scene";
    pub const VISUALIZATION_EXPORT: &str = "visualization.export";
    pub const VISUALIZATION_VALIDATE: &str = "visualization.validate";

    pub const GRAPH_TOPOLOGY: &str = "graph.topology";
    pub const GRAPH_BUILDER: &str = "graph.builder";

    pub const INTERACTION_SUBSCRIBE: &str = "interaction.subscribe";
    pub const INTERACTION_POLL: &str = "interaction.poll";

    pub const SENSOR_STREAM_SUBSCRIBE: &str = "sensor.stream.subscribe";

    pub const MOTOR_SET_PANEL: &str = "motor.set_panel";
    pub const MOTOR_SET_ZOOM: &str = "motor.set_zoom";
    pub const MOTOR_SET_MODE: &str = "motor.set_mode";
    pub const MOTOR_FIT_TO_VIEW: &str = "motor.fit_to_view";
    pub const MOTOR_NAVIGATE: &str = "motor.navigate";

    pub const MODALITY_VISUAL: &str = "modality.visual";
    pub const MODALITY_AUDIO: &str = "modality.audio";
    pub const MODALITY_TERMINAL: &str = "modality.terminal";
    pub const MODALITY_HAPTIC: &str = "modality.haptic";
    pub const MODALITY_BRAILLE: &str = "modality.braille";
    pub const MODALITY_DESCRIPTION: &str = "modality.description";

    pub const AUDIO_SYNTHESIZE: &str = "audio.synthesize";

    pub const CAPABILITIES_SENSORY: &str = "capabilities.sensory";
    pub const CAPABILITIES_SENSORY_NEGOTIATE: &str = "capabilities.sensory.negotiate";

    pub const IDENTITY_GET: &str = "identity.get";
    pub const LIFECYCLE_STATUS: &str = "lifecycle.status";
    pub const HEALTH_CHECK: &str = "health.check";
    pub const HEALTH_LIVENESS: &str = "health.liveness";
    pub const HEALTH_READINESS: &str = "health.readiness";
    pub const CAPABILITIES_LIST: &str = "capabilities.list";

    /// All capabilities as a slice for bulk registration.
    pub const ALL: &[&str] = &[
        UI_RENDER,
        UI_VISUALIZATION,
        UI_GRAPH,
        UI_TERMINAL,
        UI_AUDIO,
        VISUALIZATION_RENDER,
        VISUALIZATION_RENDER_STREAM,
        VISUALIZATION_RENDER_GRAMMAR,
        VISUALIZATION_RENDER_DASHBOARD,
        VISUALIZATION_RENDER_SCENE,
        VISUALIZATION_INTERACT,
        VISUALIZATION_INTERACT_SUBSCRIBE,
        VISUALIZATION_PROVENANCE,
        VISUALIZATION_EXPORT,
        VISUALIZATION_VALIDATE,
        GRAPH_TOPOLOGY,
        GRAPH_BUILDER,
        INTERACTION_SUBSCRIBE,
        INTERACTION_POLL,
        SENSOR_STREAM_SUBSCRIBE,
        MOTOR_SET_PANEL,
        MOTOR_SET_ZOOM,
        MOTOR_SET_MODE,
        MOTOR_FIT_TO_VIEW,
        MOTOR_NAVIGATE,
        MODALITY_VISUAL,
        MODALITY_AUDIO,
        MODALITY_TERMINAL,
        MODALITY_HAPTIC,
        MODALITY_BRAILLE,
        MODALITY_DESCRIPTION,
        AUDIO_SYNTHESIZE,
        CAPABILITIES_SENSORY,
        CAPABILITIES_SENSORY_NEGOTIATE,
        IDENTITY_GET,
        LIFECYCLE_STATUS,
        HEALTH_CHECK,
        HEALTH_LIVENESS,
        HEALTH_READINESS,
        CAPABILITIES_LIST,
    ];
}

/// Capability identifiers for discovering external services.
///
/// These are NOT primal names — they are capability strings that any
/// primal providing the service can announce.
#[expect(missing_docs, reason = "constant names mirror their semantic values")]
pub mod discovery_capabilities {
    pub const GPU_DISPATCH: &str = "gpu.dispatch";
    pub const GPU_SCIENCE_DISPATCH: &str = "science.gpu.dispatch";
    pub const DISPLAY_BACKEND: &str = "display";
    pub const SHADER_COMPILE: &str = "shader.compile";
    pub const LIFECYCLE_REGISTER: &str = "lifecycle.register";
    pub const LIFECYCLE_STATUS: &str = "lifecycle.status";
    pub const IPC_REGISTER: &str = "ipc.register";
    pub const IPC_DISCOVER: &str = "ipc.discover";
    pub const COMPUTE_DISPATCH: &str = "compute.dispatch";
}

/// Semantic IPC method names for JSON-RPC and tarpc.
#[expect(missing_docs, reason = "constant names mirror their semantic values")]
pub mod methods {
    pub const VISUALIZATION_RENDER: &str = "visualization.render";
    pub const VISUALIZATION_RENDER_STREAM: &str = "visualization.render.stream";
    pub const VISUALIZATION_RENDER_GRAMMAR: &str = "visualization.render.grammar";
    pub const VISUALIZATION_RENDER_DASHBOARD: &str = "visualization.render.dashboard";
    pub const VISUALIZATION_RENDER_SCENE: &str = "visualization.render.scene";
    pub const VISUALIZATION_VALIDATE: &str = "visualization.validate";
    pub const VISUALIZATION_EXPORT: &str = "visualization.export";
    pub const VISUALIZATION_CAPABILITIES: &str = "visualization.capabilities";
    pub const VISUALIZATION_DISMISS: &str = "visualization.dismiss";
    pub const VISUALIZATION_INTERACT_APPLY: &str = "visualization.interact.apply";
    pub const VISUALIZATION_INTERACT_PERSPECTIVES: &str = "visualization.interact.perspectives";
    pub const VISUALIZATION_INTROSPECT: &str = "visualization.introspect";
    pub const VISUALIZATION_PANELS: &str = "visualization.panels";
    pub const VISUALIZATION_SHOWING: &str = "visualization.showing";
    pub const VISUALIZATION_SESSION_LIST: &str = "visualization.session.list";
    pub const VISUALIZATION_SESSION_STATUS: &str = "visualization.session.status";
    pub const CAPABILITIES_SENSORY: &str = "capabilities.sensory";
    pub const CAPABILITIES_SENSORY_NEGOTIATE: &str = "capabilities.sensory.negotiate";
}

/// Well-known socket name prefixes for capability-based discovery.
///
/// These identify the *role* of a socket, not a specific primal.
/// Any primal providing the capability can use these names.
#[expect(missing_docs, reason = "constant names mirror their semantic values")]
pub mod socket_roles {
    pub const NEURAL_API: &str = "biomeos-neural-api";
    pub const DEVICE_MANAGEMENT: &str = "biomeos-device-management";
    pub const UI_SERVICE: &str = "biomeos-ui";
    pub const DISCOVERY_SERVICE: &str = "discovery-service";
    pub const PHYSICS_COMPUTE: &str = "physics-compute";
}

/// Well-known primal identifiers for discovery and logging.
///
/// These are the ecosystem-standard names primals announce with.
/// Used for log context and capability filtering — **never** for
/// hardcoded routing. Runtime discovery uses capability strings.
#[expect(missing_docs, reason = "constant names mirror their semantic values")]
pub mod primal_names {
    pub const PETALTONGUE: &str = "petaltongue";
    pub const BIOMEOS: &str = "biomeos";
    pub const SONGBIRD: &str = "songbird";
    pub const TOADSTOOL: &str = "toadstool";
    pub const BARRACUDA: &str = "barracuda";
    pub const CORALREEF: &str = "coralreef";
    pub const BEARDOG: &str = "beardog";
    pub const NESTGATE: &str = "nestgate";
    pub const SQUIRREL: &str = "squirrel";
    pub const RHIZOCRYPT: &str = "rhizocrypt";
    pub const SWEETGRASS: &str = "sweetgrass";
    pub const LOAMSPINE: &str = "loamspine";
    pub const SKUNKBAT: &str = "skunkbat";
    pub const SOURDOUGH: &str = "sourdough";
    pub const PLASMIDBIN: &str = "plasmidbin";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_capabilities_all_count() {
        assert_eq!(self_capabilities::ALL.len(), 40);
    }

    #[test]
    fn self_capabilities_all_unique() {
        let mut seen = std::collections::HashSet::new();
        for cap in self_capabilities::ALL {
            assert!(seen.insert(cap), "duplicate capability: {cap}");
        }
    }

    #[test]
    fn semantic_naming_convention() {
        for cap in self_capabilities::ALL {
            assert!(
                cap.contains('.'),
                "capability '{cap}' must follow domain.operation convention"
            );
        }
    }

    #[test]
    fn discovery_capabilities_follow_convention() {
        let caps = [
            discovery_capabilities::GPU_DISPATCH,
            discovery_capabilities::GPU_SCIENCE_DISPATCH,
            discovery_capabilities::DISPLAY_BACKEND,
            discovery_capabilities::LIFECYCLE_REGISTER,
            discovery_capabilities::LIFECYCLE_STATUS,
            discovery_capabilities::IPC_REGISTER,
            discovery_capabilities::IPC_DISCOVER,
            discovery_capabilities::COMPUTE_DISPATCH,
        ];
        for cap in caps {
            assert!(!cap.is_empty(), "capability constant must not be empty");
        }
    }

    #[test]
    fn methods_match_capabilities() {
        assert_eq!(
            methods::VISUALIZATION_RENDER,
            self_capabilities::VISUALIZATION_RENDER
        );
        assert_eq!(
            methods::VISUALIZATION_RENDER_STREAM,
            self_capabilities::VISUALIZATION_RENDER_STREAM
        );
    }
}
