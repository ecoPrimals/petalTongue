// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
#![allow(missing_docs)]
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
pub mod gpu_compiler;
pub mod modality;
pub mod node_id;
pub mod physics;
pub mod primitive;
pub mod render_plan;
pub mod scene_graph;
pub mod transform;
pub mod tufte;

pub use animation::{Animation, AnimationPlayer, AnimationState, Easing, Sequence};
pub use compiler::GrammarCompiler;
pub use dashboard::{
    Dashboard, DashboardConfig, DashboardLayout, build_dashboard, compose_dashboard,
};
pub use data_binding::DataBindingCompiler;
pub use domain_palette::{DivergingScale, DomainPalette, palette_for_domain};
pub use equation::EquationCompiler;
pub use gpu_compiler::{GpuCompiler, GpuDrawCommand};
pub use grammar::{
    Aesthetic, CoordinateSystem, Facet, FacetLayout, GeometryType, GrammarExpr, ScaleType,
};
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
pub use render_plan::{AxisMeta, PanelBounds, PanelMeta, RenderPlan};
pub use scene_graph::{NodeId, SceneGraph, SceneNode};
pub use transform::{Transform2D, Transform3D};
pub use tufte::{
    ChartjunkDetection, ColorAccessibility, ConstraintResult, ConstraintSeverity, DataDensity,
    DataInkRatio, LieFactor, SmallMultiplesPreference, SmallestEffectiveDifference,
    TufteConstraint, TufteReport,
};
