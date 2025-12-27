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
pub mod visual_2d;

#[cfg(feature = "native-audio")]
pub mod audio_playback;

// BingoCube adapters are now in bingoCube/adapters
// petalTongue can use them via:
// use bingocube_adapters::visual::BingoCubeVisualRenderer;
// use bingocube_adapters::audio::BingoCubeAudioRenderer;

pub use audio_export::{AudioFileGenerator, AudioFormat, AudioQuality};
pub use audio_sonification::{AudioAttributes, AudioSonificationRenderer, Instrument};

#[cfg(feature = "native-audio")]
pub use audio_playback::AudioPlaybackEngine;

pub use visual_2d::Visual2DRenderer;

// Re-export BingoCube adapters for convenience
pub use bingocube_adapters;
