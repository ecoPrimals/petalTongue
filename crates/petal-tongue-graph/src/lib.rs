//! # petal-tongue-graph
//!
//! Graph rendering implementations (visual, VR, AR, etc.)
//!
//! This crate provides concrete renderers that consume the abstract
//! graph engine and represent it visually.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
// Allow some pedantic warnings for now - will be addressed in future refactoring
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::unused_self)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::format_push_string)]
#![allow(clippy::missing_errors_doc)] // Allow for now, will add later
#![allow(clippy::missing_panics_doc)] // Allow for now, will add later

pub mod audio_export;
pub mod audio_sonification;
pub mod capability_validator;
pub mod color_utils;
pub mod visual_2d;

// REMOVED: audio_playback module (was using rodio/ALSA)
// Audio playback is now handled by:
// - AudioCanvas (in petal-tongue-ui) - pure Rust /dev/snd access
// - ToadStool (network) - discovered at runtime
// - External system audio - discovered at runtime

// BingoCube is a primalTool, discovered at runtime (not a compile-time dependency)
// ToadStool is a primalTool, discovered at runtime (not a compile-time dependency)
// ALSA is an external system, discovered at runtime (not a compile-time dependency)

pub use audio_export::{AudioFileGenerator, AudioFormat, AudioQuality};
pub use audio_sonification::{AudioAttributes, AudioSonificationRenderer, Instrument};
pub use color_utils::{hsv_to_rgb, lerp_hsv, rgb_to_hsv};
pub use visual_2d::Visual2DRenderer;

// REMOVED: AudioPlaybackEngine export (was requiring ALSA/rodio)
// Use AudioCanvas from petal-tongue-ui for pure Rust audio playback
