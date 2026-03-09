// SPDX-License-Identifier: AGPL-3.0-only
//! Simple audio test - generates and saves a WAV file

use petal_tongue_ui::audio_pure_rust::{Waveform, export_wav, generate_tone};

fn main() {
    println!("🔊 Generating test audio...");

    // Generate 440Hz (A note) sine wave for 0.5 seconds
    let samples = generate_tone(0.5, 440.0, Waveform::Sine, 0.7);

    // Export to WAV
    let wav_bytes = export_wav(&samples);

    // Save to file
    let path = "/tmp/petaltongue_test.wav";
    std::fs::write(path, wav_bytes).expect("Failed to write WAV file");

    println!("✅ Generated {} samples", samples.len());
    println!("💾 Saved to: {path}");
    println!("🎵 Frequency: 440Hz (A note)");
    println!("⏱️  Duration: 0.5s");
    println!();
    println!("Play with: aplay {path} (or paplay, mpv, etc.)");
}
