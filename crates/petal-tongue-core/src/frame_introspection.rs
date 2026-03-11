// SPDX-License-Identifier: AGPL-3.0-only
//! Frame introspection -- content-level self-awareness for petalTongue.
//!
//! `RenderingAwareness` tracks *that* frames were sent and acknowledged.
//! `FrameIntrospection` tracks *what* each frame contains: which panels are
//! visible, what data is bound, and what interactions are possible.
//!
//! Together they close the proprioceptive loop: petalTongue knows both that
//! its arm moved AND what its hand is touching.

use crate::interaction::perspective::OutputModality;
use crate::rendering_awareness::PanelId;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Snapshot of what a single frame contains.
///
/// Produced by the UI layer after each `update()` cycle and fed into
/// `RenderingAwareness::record_frame_content()`.
#[derive(Debug, Clone)]
pub struct FrameIntrospection {
    /// Which rendering frame this snapshot corresponds to.
    pub frame_id: u64,
    /// When this snapshot was taken.
    pub timestamp: Instant,
    /// All panels and their visibility state.
    pub visible_panels: Vec<PanelSnapshot>,
    /// Data objects currently bound to visual elements.
    pub bound_data: Vec<BoundDataObject>,
    /// Interactions that the user could perform right now.
    pub possible_interactions: Vec<InteractionCapability>,
    /// Which output modalities are currently active.
    pub active_modalities: Vec<OutputModality>,
}

impl FrameIntrospection {
    /// Create an empty introspection for the given frame.
    #[must_use]
    pub fn empty(frame_id: u64) -> Self {
        Self {
            frame_id,
            timestamp: Instant::now(),
            visible_panels: Vec::new(),
            bound_data: Vec::new(),
            possible_interactions: Vec::new(),
            active_modalities: Vec::new(),
        }
    }

    /// Whether a panel of the given kind is visible.
    #[must_use]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "ergonomic for callers passing enum literals"
    )]
    pub fn is_panel_visible(&self, kind: PanelKind) -> bool {
        self.visible_panels
            .iter()
            .any(|p| p.visible && p.kind == kind)
    }

    /// Whether a specific data object is currently bound to any visual element.
    #[must_use]
    pub fn is_showing_data(&self, data_id: &str) -> bool {
        self.bound_data.iter().any(|b| b.data_object_id == data_id)
    }

    /// Count of visible panels.
    #[must_use]
    pub fn visible_panel_count(&self) -> usize {
        self.visible_panels.iter().filter(|p| p.visible).count()
    }

    /// All visible panel kinds.
    #[must_use]
    pub fn visible_panel_kinds(&self) -> Vec<PanelKind> {
        self.visible_panels
            .iter()
            .filter(|p| p.visible)
            .map(|p| p.kind.clone())
            .collect()
    }
}

/// Snapshot of a single panel's state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelSnapshot {
    /// Panel identity in the motor command system.
    pub id: PanelId,
    /// Semantic kind of this panel.
    pub kind: PanelKind,
    /// Whether the panel is currently visible.
    pub visible: bool,
    /// What data source feeds this panel (if any).
    pub data_source: Option<String>,
    /// Approximate number of widgets in the panel.
    pub widget_count: usize,
}

impl PanelSnapshot {
    /// Convenience constructor for a visible panel.
    #[must_use]
    pub const fn visible(id: PanelId, kind: PanelKind) -> Self {
        Self {
            id,
            kind,
            visible: true,
            data_source: None,
            widget_count: 0,
        }
    }

    /// Convenience constructor for a hidden panel.
    #[must_use]
    pub const fn hidden(id: PanelId, kind: PanelKind) -> Self {
        Self {
            id,
            kind,
            visible: false,
            data_source: None,
            widget_count: 0,
        }
    }

    /// Set the data source.
    #[must_use]
    pub fn with_data_source(mut self, source: impl Into<String>) -> Self {
        self.data_source = Some(source.into());
        self
    }

    /// Set widget count.
    #[must_use]
    pub const fn with_widget_count(mut self, count: usize) -> Self {
        self.widget_count = count;
        self
    }
}

/// Semantic category of a UI panel.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelKind {
    /// Top menu bar.
    TopMenu,
    /// Left sidebar controls (graph stats, layout, animation toggle).
    Controls,
    /// Audio sonification panel.
    AudioSonification,
    /// Capability status overview.
    CapabilityStatus,
    /// System metrics dashboard.
    Dashboard,
    /// Trust relationship dashboard.
    TrustDashboard,
    /// Proprioception / SAME DAVE panel.
    Proprioception,
    /// Neural API metrics dashboard.
    Metrics,
    /// Interactive graph builder.
    GraphBuilder,
    /// Main graph canvas (central panel).
    GraphCanvas,
    /// Primal detail inspector.
    PrimalDetails,
    /// Awakening overlay.
    Awakening,
    /// Accessibility settings.
    Accessibility,
    /// Named custom panel from a plugin or tool.
    Custom(String),
}

/// A data object bound to a visual element in the current frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundDataObject {
    /// Which panel contains this binding.
    pub panel_id: PanelId,
    /// The data object identifier (perspective-invariant).
    pub data_object_id: String,
    /// How the data is represented.
    pub binding_type: BindingType,
}

/// How a data object is visually represented.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingType {
    /// A node in the graph canvas.
    GraphNode,
    /// An edge in the graph canvas.
    GraphEdge,
    /// A row or item in a list/table.
    ListItem,
    /// A chart data point.
    ChartPoint,
    /// A metric value display.
    MetricValue,
    /// A label or text annotation.
    Label,
}

/// An interaction the user could perform in the current state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionCapability {
    /// Which panel supports this interaction.
    pub panel_id: PanelId,
    /// What kind of interaction is possible.
    pub intent: InteractionKind,
    /// What the interaction targets (if specific).
    pub target: Option<String>,
}

/// The kind of interaction available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionKind {
    /// User can select an element.
    Select,
    /// User can inspect/drill into an element.
    Inspect,
    /// User can navigate/zoom/pan.
    Navigate,
    /// User can toggle a panel's visibility.
    TogglePanel,
    /// User can change a setting.
    Configure,
    /// User can trigger a refresh.
    Refresh,
    /// User can export data.
    Export,
}

/// Content awareness: the "what" counterpart to the "that" of motor/sensory tracking.
///
/// Stored inside `RenderingAwareness` to unify frame tracking with content tracking.
#[derive(Debug)]
pub struct ContentAwareness {
    current: Option<FrameIntrospection>,
    history_len: usize,
    total_introspections: u64,
}

impl ContentAwareness {
    /// Create a new content awareness tracker.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            current: None,
            history_len: 0,
            total_introspections: 0,
        }
    }

    /// Record a new frame's content.
    pub fn record(&mut self, introspection: FrameIntrospection) {
        self.current = Some(introspection);
        self.history_len += 1;
        self.total_introspections += 1;
    }

    /// The most recent frame introspection.
    #[must_use]
    pub const fn current(&self) -> Option<&FrameIntrospection> {
        self.current.as_ref()
    }

    /// Whether any content has ever been recorded.
    #[must_use]
    pub const fn has_content(&self) -> bool {
        self.current.is_some()
    }

    /// Total number of introspections recorded.
    #[must_use]
    pub const fn total_introspections(&self) -> u64 {
        self.total_introspections
    }

    /// Panels currently visible.
    #[must_use]
    pub fn visible_panels(&self) -> Vec<PanelId> {
        self.current
            .as_ref()
            .map(|f| {
                f.visible_panels
                    .iter()
                    .filter(|p| p.visible)
                    .map(|p| p.id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Whether specific data is currently shown.
    #[must_use]
    pub fn is_showing_data(&self, data_id: &str) -> bool {
        self.current
            .as_ref()
            .is_some_and(|f| f.is_showing_data(data_id))
    }
}

impl Default for ContentAwareness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_introspection() {
        let frame = FrameIntrospection::empty(42);
        assert_eq!(frame.frame_id, 42);
        assert!(frame.visible_panels.is_empty());
        assert!(frame.bound_data.is_empty());
        assert_eq!(frame.visible_panel_count(), 0);
        assert!(!frame.is_showing_data("anything"));
    }

    #[test]
    fn panel_visibility() {
        let mut frame = FrameIntrospection::empty(1);
        frame
            .visible_panels
            .push(PanelSnapshot::visible(PanelId::TopMenu, PanelKind::TopMenu));
        frame.visible_panels.push(PanelSnapshot::hidden(
            PanelId::AudioPanel,
            PanelKind::AudioSonification,
        ));

        assert!(frame.is_panel_visible(PanelKind::TopMenu));
        assert!(!frame.is_panel_visible(PanelKind::AudioSonification));
        assert_eq!(frame.visible_panel_count(), 1);
    }

    #[test]
    fn data_binding_query() {
        let mut frame = FrameIntrospection::empty(1);
        frame.bound_data.push(BoundDataObject {
            panel_id: PanelId::Custom("canvas".into()),
            data_object_id: "primal-abc".into(),
            binding_type: BindingType::GraphNode,
        });

        assert!(frame.is_showing_data("primal-abc"));
        assert!(!frame.is_showing_data("primal-xyz"));
    }

    #[test]
    fn content_awareness_lifecycle() {
        let mut content = ContentAwareness::new();
        assert!(!content.has_content());
        assert!(content.visible_panels().is_empty());
        assert!(!content.is_showing_data("x"));

        let mut frame = FrameIntrospection::empty(1);
        frame
            .visible_panels
            .push(PanelSnapshot::visible(PanelId::TopMenu, PanelKind::TopMenu));
        frame.bound_data.push(BoundDataObject {
            panel_id: PanelId::TopMenu,
            data_object_id: "node-1".into(),
            binding_type: BindingType::GraphNode,
        });

        content.record(frame);
        assert!(content.has_content());
        assert_eq!(content.visible_panels().len(), 1);
        assert!(content.is_showing_data("node-1"));
        assert_eq!(content.total_introspections(), 1);
    }

    #[test]
    fn panel_snapshot_builder() {
        let snap = PanelSnapshot::visible(PanelId::SystemDashboard, PanelKind::Dashboard)
            .with_data_source("proc_stats")
            .with_widget_count(12);

        assert!(snap.visible);
        assert_eq!(snap.data_source.as_deref(), Some("proc_stats"));
        assert_eq!(snap.widget_count, 12);
    }

    #[test]
    fn visible_panel_kinds() {
        let mut frame = FrameIntrospection::empty(1);
        frame
            .visible_panels
            .push(PanelSnapshot::visible(PanelId::TopMenu, PanelKind::TopMenu));
        frame.visible_panels.push(PanelSnapshot::visible(
            PanelId::LeftSidebar,
            PanelKind::Controls,
        ));
        frame.visible_panels.push(PanelSnapshot::hidden(
            PanelId::AudioPanel,
            PanelKind::AudioSonification,
        ));

        let kinds = frame.visible_panel_kinds();
        assert_eq!(kinds.len(), 2);
        assert!(kinds.contains(&PanelKind::TopMenu));
        assert!(kinds.contains(&PanelKind::Controls));
    }

    #[test]
    fn test_panel_snapshot_hidden() {
        let snap = PanelSnapshot::hidden(PanelId::Proprioception, PanelKind::Proprioception);
        assert!(!snap.visible);
        assert_eq!(snap.kind, PanelKind::Proprioception);
    }

    #[test]
    fn test_content_awareness_default() {
        let content = ContentAwareness::default();
        assert!(!content.has_content());
        assert_eq!(content.total_introspections(), 0);
    }

    #[test]
    fn test_content_awareness_record_overwrites() {
        let mut content = ContentAwareness::new();
        content.record(FrameIntrospection::empty(1));
        content.record(FrameIntrospection::empty(2));
        assert_eq!(content.total_introspections(), 2);
        assert_eq!(content.current().expect("current").frame_id, 2);
    }

    #[test]
    fn test_possible_interactions_and_active_modalities() {
        let mut frame = FrameIntrospection::empty(1);
        frame.possible_interactions.push(InteractionCapability {
            panel_id: PanelId::GraphStats,
            intent: InteractionKind::Select,
            target: Some("node-1".to_string()),
        });
        frame.active_modalities.push(OutputModality::Gui);
        assert_eq!(frame.possible_interactions.len(), 1);
        assert_eq!(frame.active_modalities.len(), 1);
    }

    #[test]
    fn test_binding_type_variants() {
        assert_eq!(BindingType::GraphNode, BindingType::GraphNode);
        assert_eq!(BindingType::ListItem, BindingType::ListItem);
        assert_eq!(BindingType::ChartPoint, BindingType::ChartPoint);
    }

    #[test]
    fn test_interaction_kind_variants() {
        assert_eq!(InteractionKind::Navigate, InteractionKind::Navigate);
        assert_eq!(InteractionKind::Export, InteractionKind::Export);
    }

    #[test]
    fn test_panel_kind_custom() {
        let kind = PanelKind::Custom("my_panel".to_string());
        assert_eq!(kind, PanelKind::Custom("my_panel".to_string()));
    }

    #[test]
    fn test_is_panel_visible_hidden_panel_same_kind() {
        let mut frame = FrameIntrospection::empty(1);
        frame
            .visible_panels
            .push(PanelSnapshot::hidden(PanelId::TopMenu, PanelKind::TopMenu));
        assert!(!frame.is_panel_visible(PanelKind::TopMenu));
    }

    #[test]
    fn test_is_panel_visible_no_match() {
        let mut frame = FrameIntrospection::empty(1);
        frame
            .visible_panels
            .push(PanelSnapshot::visible(PanelId::TopMenu, PanelKind::TopMenu));
        assert!(!frame.is_panel_visible(PanelKind::GraphCanvas));
    }

    #[test]
    fn test_panel_snapshot_with_data_source() {
        let snap =
            PanelSnapshot::visible(PanelId::Custom("p".into()), PanelKind::Custom("x".into()))
                .with_data_source("source");
        assert_eq!(snap.data_source.as_deref(), Some("source"));
    }

    #[test]
    fn test_panel_snapshot_with_widget_count() {
        let snap =
            PanelSnapshot::visible(PanelId::TopMenu, PanelKind::TopMenu).with_widget_count(5);
        assert_eq!(snap.widget_count, 5);
    }

    #[test]
    fn test_interaction_capability_construction() {
        let cap = InteractionCapability {
            panel_id: PanelId::GraphStats,
            intent: InteractionKind::Inspect,
            target: None,
        };
        assert_eq!(cap.intent, InteractionKind::Inspect);
        assert!(cap.target.is_none());
    }

    #[test]
    fn test_binding_type_all_variants() {
        assert_eq!(BindingType::GraphEdge, BindingType::GraphEdge);
        assert_eq!(BindingType::MetricValue, BindingType::MetricValue);
        assert_eq!(BindingType::Label, BindingType::Label);
    }

    #[test]
    fn test_panel_kind_all_variants() {
        assert_eq!(PanelKind::TopMenu, PanelKind::TopMenu);
        assert_eq!(PanelKind::Controls, PanelKind::Controls);
        assert_eq!(PanelKind::GraphBuilder, PanelKind::GraphBuilder);
        assert_eq!(PanelKind::GraphCanvas, PanelKind::GraphCanvas);
        assert_eq!(PanelKind::PrimalDetails, PanelKind::PrimalDetails);
        assert_eq!(PanelKind::Awakening, PanelKind::Awakening);
        assert_eq!(PanelKind::Accessibility, PanelKind::Accessibility);
    }
}
