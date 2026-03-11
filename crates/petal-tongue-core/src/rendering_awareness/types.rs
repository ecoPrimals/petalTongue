// SPDX-License-Identifier: AGPL-3.0-only
//! Types and enums for rendering awareness.

/// Identifier for a UI panel (used by efferent motor commands).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PanelId {
    /// Left sidebar (controls, graph stats, animation)
    LeftSidebar,
    /// Right sidebar (audio, dashboard, trust)
    RightSidebar,
    /// Top menu bar
    TopMenu,
    /// System metrics dashboard
    SystemDashboard,
    /// Audio controls panel
    AudioPanel,
    /// Trust relationship dashboard
    TrustDashboard,
    /// Proprioception / SAME DAVE panel
    Proprioception,
    /// Graph statistics overlay
    GraphStats,
    /// Named custom panel
    Custom(String),
}

/// Motor command types — efferent signals that change UI state.
///
/// These flow through efferent channels from the proprioception core
/// to the UI effectors (panels, renderer, layout engine).
#[derive(Debug, Clone)]
pub enum MotorCommand {
    // === Rendering (existing) ===
    /// Render a specific frame
    RenderFrame {
        /// Frame identifier
        frame_id: u64,
    },
    /// Update display without new frame
    UpdateDisplay,
    /// Clear the display
    ClearDisplay,

    // === Panel visibility (efferent) ===
    /// Show or hide a specific panel
    SetPanelVisibility {
        /// Which panel to control
        panel: PanelId,
        /// Whether it should be visible
        visible: bool,
    },

    // === Camera / viewport (efferent) ===
    /// Set the zoom level on the graph renderer
    SetZoom {
        /// Zoom level (1.0 = default)
        level: f32,
    },
    /// Fit all nodes into the viewport
    FitToView,
    /// Center the viewport on a specific node
    Navigate {
        /// Node ID to center on
        target_node: String,
    },
    /// Select (or deselect) a node
    SelectNode {
        /// Node ID, or None to deselect
        node_id: Option<String>,
    },

    // === Layout (efferent) ===
    /// Change the graph layout algorithm
    SetLayout {
        /// Algorithm name (e.g. "ForceDirected", "Grid", "Radial")
        algorithm: String,
    },

    // === Mode / profile (efferent) ===
    /// Switch to a named mode (applies a preset bundle of commands)
    SetMode {
        /// Mode name (e.g. "clinical", "developer", "presentation")
        mode: String,
    },

    // === Startup / lifecycle (efferent) ===
    /// Enable or disable the awakening overlay
    SetAwakening {
        /// Whether the overlay should be active
        enabled: bool,
    },
    /// Load a new scenario from a file path
    LoadScenario {
        /// Path to the scenario JSON file
        path: String,
    },

    // === Continuous mode (game loop) ===
    /// Enable or disable 60 Hz continuous rendering (game loop)
    SetContinuousMode {
        /// Whether continuous mode is active
        enabled: bool,
    },
    /// Enable or disable physics simulation in the tick loop
    SetPhysics {
        /// Whether physics stepping is active
        enabled: bool,
    },
    /// Enable or disable scene animation in the tick loop
    SetSceneAnimation {
        /// Whether scene animation stepping is active
        enabled: bool,
    },
}

/// Command ID (unique identifier for tracking)
pub type CommandId = u64;

/// Complete self-assessment of the central nervous system
#[derive(Debug, Clone)]
pub struct SelfAssessment {
    // Motor
    /// Whether the system can render output (motor capability)
    pub can_render: bool,
    /// Total number of frames sent (motor activity)
    pub frames_sent: u64,

    // Sensory
    /// Whether the system can receive input (sensory capability)
    pub can_sense: bool,
    /// Number of frames with confirmed sensory feedback
    pub frames_confirmed: u64,

    // Bidirectional
    /// Whether the bidirectional loop is complete
    pub is_complete_loop: bool,
    /// Rate of sensory confirmation (`frames_confirmed` / `frames_sent`)
    pub confirmation_rate: f32,

    // User state
    /// Current visibility state from user perspective
    pub user_visibility: VisibilityState,
    /// Current interactivity state from user perspective
    pub user_interactivity: InteractivityState,

    // Health
    /// Whether the display substrate is responsive
    pub substrate_responsive: bool,
}

impl SelfAssessment {
    /// Check if everything is working
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.can_render && self.can_sense && self.is_complete_loop && self.substrate_responsive
    }

    /// Get overall health percentage
    #[must_use]
    pub fn health_percentage(&self) -> f32 {
        let mut score = 0.0;

        if self.can_render {
            score += 20.0;
        }
        if self.can_sense {
            score += 20.0;
        }
        if self.is_complete_loop {
            score += 30.0;
        }
        if self.substrate_responsive {
            score += 10.0;
        }

        score += (self.confirmation_rate * 0.2).min(20.0);

        score
    }
}

/// User visibility state - confidence that user can see output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibilityState {
    /// High confidence user can see output (>90% confirmation)
    Confirmed,
    /// Likely user can see output (>50% confirmation)
    Probable,
    /// Uncertain if user can see output (>0% confirmation)
    Uncertain,
    /// No confirmation of visibility
    Unknown,
}

/// User interactivity state - recency of user interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractivityState {
    /// User actively interacting (interacted <5s ago)
    Active,
    /// Recent user interaction (interacted <30s ago)
    Recent,
    /// User idle (interacted >30s ago)
    Idle,
    /// No user interaction confirmed
    Unconfirmed,
}

/// Validation health metrics
#[derive(Debug, Clone)]
pub struct ValidationHealth {
    /// Is validation pipeline healthy?
    pub healthy: bool,
    /// Percentage of frames confirmed
    pub confirmation_rate: f32,
    /// Number of unconfirmed frames
    pub unconfirmed_count: usize,
}

/// Rendering metrics - quantitative feedback on rendering effectiveness
#[derive(Debug, Clone, Default)]
pub struct RenderingMetrics {
    /// Total rendering commands sent to output
    pub commands_sent: u64,
    /// Number of frames confirmed as visible
    pub frames_confirmed: u64,
    /// Number of user interactions detected
    pub user_interactions: u64,
}
