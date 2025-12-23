//! # petal-tongue-graph
//!
//! Graph rendering implementations (visual, VR, AR, etc.)
//!
//! This crate provides concrete renderers that consume the abstract
//! graph engine and represent it visually.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)] // Allow for now, will add later
#![allow(clippy::missing_panics_doc)] // Allow for now, will add later

pub mod visual_2d;
// Audio playback temporarily disabled - needs ALSA libraries
// pub mod audio_playback;
pub mod audio_sonification;

pub use visual_2d::Visual2DRenderer;
pub use audio_sonification::{AudioSonificationRenderer, Instrument, AudioAttributes};
