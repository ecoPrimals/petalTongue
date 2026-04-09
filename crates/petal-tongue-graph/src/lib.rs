// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! # petal-tongue-graph
//!
//! Graph rendering implementations (visual, VR, AR, etc.)
//!
//! This crate provides concrete renderers that consume the abstract
//! graph engine and represent it visually.

#![expect(
    clippy::format_push_string,
    reason = "DOT/SVG builders use incremental string construction"
)]

pub mod audio_export;
pub mod audio_export_error;
pub mod audio_sonification;
pub mod capability_validator;
pub mod color_utils;

#[cfg(feature = "egui-render")]
pub mod chart_renderer;
#[cfg(feature = "egui-render")]
pub mod clinical_theme;
#[cfg(feature = "egui-render")]
pub mod domain_theme;
#[cfg(feature = "egui-render")]
pub mod visual_2d;

// BingoCube is a primalTool, discovered at runtime (not a compile-time dependency)
// ToadStool is a primalTool, discovered at runtime (not a compile-time dependency)
// ALSA is an external system, discovered at runtime (not a compile-time dependency)

pub use audio_export::{AudioFileGenerator, AudioFormat, AudioQuality};
pub use audio_export_error::AudioExportError;
pub use audio_sonification::{AudioAttributes, AudioSonificationRenderer, Instrument};
#[cfg(feature = "egui-render")]
pub use chart_renderer::{NodeDetail, draw_channel, draw_node_detail};
#[cfg(feature = "egui-render")]
pub use clinical_theme::{
    BG_CARD, BG_PANEL, CRITICAL, HEALTHY, INFO, POPULATION, TEXT_DIM, TEXT_PRIMARY, WARNING,
    health_color,
};
pub use color_utils::{hsv_to_rgb, lerp_hsv, rgb_to_hsv};
#[cfg(feature = "egui-render")]
pub use visual_2d::Visual2DRenderer;

// Use AudioCanvas from petal-tongue-ui for pure Rust audio playback
