// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![expect(
    missing_docs,
    reason = "scene graph modules evolving — docs tracked for incremental completion"
)]
//! Declarative scene graph with animation, grammar compilation, and modality dispatch.
//!
//! This crate is the intermediate representation between data sources and rendering
//! backends. It replaces imperative draw calls with a declarative scene graph that
//! can be compiled to any output modality (egui, ratatui, audio, SVG, GPU).
//!
//! ## Architecture
//!
//! ```text
//! Data Sources ──► GrammarExpr ──► Tufte Constraints ──► SceneGraph
//!                                                           │
//!                                                      Animation
//!                                                           │
//!                                              ┌────────────┼────────────┐
//!                                              ▼            ▼            ▼
//!                                         EguiCompiler  SvgCompiler  AudioCompiler
//!                                              │            │            │
//!                                         InversePipeline   ...         ...
//! ```

pub mod animation;
pub mod audio_synthesis;
pub mod compiler;
pub mod dashboard;
pub mod data_binding;
pub mod domain_palette;
pub mod equation;
pub mod grammar;
pub mod math;
/// Backward compatibility: re-export math module.
pub mod math_objects {
    pub use super::math::*;
}
pub mod game_loop;
pub mod golden_tests;
pub mod gpu_compiler;
pub mod haptic_sequence;
pub mod modality;
pub mod node_id;
pub mod physics;
pub mod primitive;
pub mod provenance;
pub mod render_plan;
pub mod scene_graph;
pub mod soundscape;
pub mod sprite;
pub mod transform;
pub mod tufte;

pub use animation::{Animation, AnimationPlayer, AnimationState, Easing, Sequence};
pub use audio_synthesis::{
    AudioProvenance, AudioProvenanceMap, AudioSynthesizer, synthesize_with_provenance,
};
pub use compiler::GrammarCompiler;
pub use dashboard::{
    Dashboard, DashboardConfig, DashboardLayout, build_dashboard, compose_dashboard,
};
pub use data_binding::DataBindingCompiler;
pub use domain_palette::{DivergingScale, DomainPalette, palette_for_domain};
pub use equation::EquationCompiler;
pub use golden_tests::{GoldenTest, GoldenTestResult, TufteVerificationResult};
pub use gpu_compiler::{
    GpuCommandWithProvenance, GpuCompiler, GpuDrawCommand, GpuProvenanceEntry, GpuProvenanceMap,
};
pub use grammar::{
    Aesthetic, CoordinateSystem, Facet, FacetLayout, GeometryType, GrammarExpr, ScaleType,
};
pub use haptic_sequence::{HapticPulse, HapticSequence};
pub use math::{Axes, FunctionPlot, MathObject, NumberLine, ParametricCurve, VectorField};
pub use modality::{
    AudioParam, BrailleCell, BrailleCompiler, HapticCommand, HapticCompiler, HapticPattern,
    ModalityCompiler, ModalityOutput,
};
pub use physics::{CollisionShape, PhysicsBody, PhysicsWorld};
pub use primitive::{
    AnchorPoint, BezierSegment, Color, FillRule, LineCap, LineJoin, MeshVertex, Primitive,
    StrokeStyle,
};
pub use provenance::{ProvenanceBuffer, ProvenanceRenderer};
pub use render_plan::{AxisMeta, PanelBounds, PanelMeta, RenderPlan};
pub use scene_graph::{NodeId, SceneGraph, SceneNode};
pub use transform::{Transform2D, Transform3D};
pub use tufte::{
    ChartjunkDetection, ColorAccessibility, ConstraintResult, ConstraintSeverity, DataDensity,
    DataInkRatio, LieFactor, SmallMultiplesPreference, SmallestEffectiveDifference,
    TufteConstraint, TufteReport,
};
