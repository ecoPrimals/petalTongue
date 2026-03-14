// SPDX-License-Identifier: AGPL-3.0-only
//! Shared playback utilities for legacy audio providers.
//!
//! Uses Audio Canvas (direct hardware access) for sample and file playback.

use std::path::Path;
use tracing::info;

/// Play audio samples using Audio Canvas (direct hardware!)
///
/// EVOLVED: Like WGPU for graphics - direct device access!
pub fn play_samples(samples: &[f32], _sample_rate: u32) -> Result<(), String> {
    use crate::audio_canvas::AudioCanvas;

    info!(
        "🎨 Playing {} samples via Audio Canvas (100% pure Rust!)",
        samples.len()
    );

    // Open audio canvas (direct hardware access!)
    let mut canvas =
        AudioCanvas::open_default().map_err(|e| format!("Failed to open audio canvas: {e}"))?;

    // Write samples directly to hardware!
    canvas
        .write_samples(samples)
        .map_err(|e| format!("Failed to write samples: {e}"))?;

    info!("✅ Audio playback complete (Audio Canvas)");

    Ok(())
}

/// Play audio file using Audio Canvas + symphonia (100% pure Rust!)
pub fn play_file(path: &Path) -> Result<(), String> {
    use crate::audio_canvas::AudioCanvas;
    use std::fs;

    info!("🎨 Playing audio file: {} (Audio Canvas)", path.display());

    // Read file
    let data = fs::read(path).map_err(|e| format!("Failed to read audio file: {e}"))?;

    // Decode with symphonia (pure Rust!)
    let decoded = crate::startup_audio::decode_audio_symphonia(&data)
        .map_err(|e| format!("Failed to decode audio: {e}"))?;

    // Open audio canvas
    let mut canvas =
        AudioCanvas::open_default().map_err(|e| format!("Failed to open audio canvas: {e}"))?;

    // Write samples directly to hardware!
    canvas
        .write_samples(&decoded.samples)
        .map_err(|e| format!("Failed to write samples: {e}"))?;

    info!("✅ Audio file playback complete (Audio Canvas)");

    Ok(())
}
