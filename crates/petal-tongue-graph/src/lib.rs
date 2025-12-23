//! # petal-tongue-graph
//!
//! Graph rendering implementations (visual, VR, AR, etc.)
//!
//! This crate provides concrete renderers that consume the abstract
//! graph engine and represent it visually.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod visual_2d;

pub use visual_2d::Visual2DRenderer;
