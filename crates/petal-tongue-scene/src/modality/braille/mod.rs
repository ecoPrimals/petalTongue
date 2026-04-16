// SPDX-License-Identifier: AGPL-3.0-or-later
//! Braille modality: types, rasterization, and scene compilation.

mod raster;
mod types;

#[cfg(test)]
mod tests;

pub use raster::BrailleCompiler;
pub use types::BrailleCell;
