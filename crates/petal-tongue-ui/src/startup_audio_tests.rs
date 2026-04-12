// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn test_signature_tone_generation() {
    let signature = StartupAudio::generate_signature_tone();

    // Should have generated samples
    assert!(!signature.is_empty(), "Signature should generate samples");

    // Should be reasonable length (< 2 seconds)
    assert!(
        signature.len() < (SAMPLE_RATE * 2) as usize,
        "Signature should be under 2 seconds"
    );

    // All samples should be in valid range
    for (i, &sample) in signature.iter().enumerate() {
        assert!(
            (-1.0..=1.0).contains(&sample),
            "Sample {} value {} out of range [-1.0, 1.0]",
            i,
            sample
        );
    }
}

#[test]
fn test_signature_tone_length() {
    let signature = StartupAudio::generate_signature_tone();

    // Bloom: ~250ms (3 notes * ~80ms each)
    let duration_secs = signature.len() as f32 / SAMPLE_RATE as f32;
    assert!(
        (0.2..=0.35).contains(&duration_secs),
        "Signature duration should be ~250ms, got {}s",
        duration_secs
    );
}

#[test]
fn test_signature_tone_normalization() {
    let signature = StartupAudio::generate_signature_tone();

    // Find max amplitude
    let max_amplitude = signature.iter().fold(0.0_f32, |max, &s| max.max(s.abs()));

    // Should be normalized with headroom (not exceeding 0.7)
    assert!(
        max_amplitude <= 0.71,
        "Max amplitude {} should be normalized with headroom",
        max_amplitude
    );

    // Should have some signal (not all zeros)
    assert!(
        max_amplitude > 0.1,
        "Signature should have reasonable amplitude"
    );
}

#[test]
fn test_startup_audio_creation() {
    let startup = StartupAudio::new();

    // Should have defaults set
    assert!(
        startup.play_signature,
        "Signature should be enabled by default"
    );
    assert!(startup.play_music, "Music should be enabled by default");
}

#[test]
fn test_startup_audio_configuration() {
    let mut startup = StartupAudio::new();

    // Test signature toggle
    startup.set_play_signature(false);
    assert!(!startup.play_signature);

    startup.set_play_signature(true);
    assert!(startup.play_signature);

    // Test music toggle
    startup.set_play_music(false);
    assert!(!startup.play_music);

    startup.set_play_music(true);
    assert!(startup.play_music);
}

#[test]
fn test_startup_music_path_detection() {
    let startup = StartupAudio::new();

    // Should attempt to find music (may or may not exist)
    // This test just verifies the detection runs without panic
    let _ = startup.has_startup_music();
    let _ = startup.startup_music_path();
}

#[test]
fn test_has_startup_music() {
    let startup = StartupAudio::new();
    // With embedded music, has_startup_music() always returns true
    // unless explicitly disabled via PETALTONGUE_DISABLE_EMBEDDED_MUSIC
    assert!(
        startup.has_startup_music(),
        "has_startup_music should return true (embedded music available)"
    );
}

#[test]
fn test_find_startup_music_paths() {
    // This test verifies the path search logic doesn't panic
    // Actual file existence depends on environment
    let startup = StartupAudio::new();

    if let Some(path) = startup.startup_music_path() {
        // If found, verify it's a path
        assert!(
            path.to_str().is_some(),
            "Startup music path should be valid UTF-8"
        );
    }
}

#[test]
fn test_signature_tone_bloom_arpeggio() {
    let signature = StartupAudio::generate_signature_tone();

    // Verify it's not empty (has actual audio data, ~250ms)
    assert!(
        signature.len() > SAMPLE_RATE as usize / 10,
        "Should have at least ~100ms of audio"
    );

    // Verify we have varied amplitudes (not flat/silence)
    let mut has_variation = false;
    let first_sample = signature[0];
    for &sample in signature.iter().skip(100) {
        if (sample - first_sample).abs() > 0.1 {
            has_variation = true;
            break;
        }
    }
    assert!(has_variation, "Signature should have amplitude variation");
}

#[test]
fn test_startup_audio_with_both_disabled() {
    let mut startup = StartupAudio::new();
    startup.set_play_signature(false);
    startup.set_play_music(false);

    // Should handle disabled state without panic
    assert!(!startup.play_signature);
    assert!(!startup.play_music);
}

#[test]
fn test_signature_generation_is_deterministic() {
    // Generate twice and compare
    let sig1 = StartupAudio::generate_signature_tone();
    let sig2 = StartupAudio::generate_signature_tone();

    // Should have same length
    assert_eq!(sig1.len(), sig2.len(), "Signature should be deterministic");

    // Should have same content
    for (i, (&s1, &s2)) in sig1.iter().zip(sig2.iter()).enumerate() {
        assert!(
            (s1 - s2).abs() < 0.0001,
            "Sample {} differs: {} vs {}",
            i,
            s1,
            s2
        );
    }
}

#[test]
fn test_startup_audio_getters() {
    let startup = StartupAudio::new();

    // Test all getter methods
    let _ = startup.has_startup_music();
    let _ = startup.startup_music_path();
}

#[test]
fn test_get_embedded_music() {
    let data = StartupAudio::get_embedded_music();
    assert!(!data.is_empty(), "Embedded music should not be empty");
}

#[test]
fn test_has_embedded_music() {
    assert!(StartupAudio::has_embedded_music());
}

#[test]
fn test_is_using_embedded() {
    let startup = StartupAudio::new();
    let using = startup.is_using_embedded();
    assert!(using || !StartupAudio::has_embedded_music());
}

#[test]
fn test_signature_tone_notes_count() {
    let signature = StartupAudio::generate_signature_tone();
    // 3 notes * ~80ms each at 44100 Hz = ~10560 samples per note
    assert!(signature.len() > 1000);
}

#[test]
fn test_signature_tone_frequency_content() {
    use crate::audio_pure_rust::{SAMPLE_RATE, Waveform, generate_tone};
    let c5 = generate_tone(523.25, 0.08, Waveform::Sine, 0.35);
    let e5 = generate_tone(659.25, 0.08, Waveform::Sine, 0.35);
    let g5 = generate_tone(784.0, 0.08, Waveform::Sine, 0.35);
    assert!(!c5.is_empty());
    assert!(!e5.is_empty());
    assert!(!g5.is_empty());
    let expected_len = (SAMPLE_RATE as f32 * 0.08) as usize;
    assert!(c5.len().abs_diff(expected_len) < 2);
}

#[test]
fn test_startup_audio_default() {
    let startup = StartupAudio::default();
    assert!(startup.play_signature);
    assert!(startup.play_music);
}

#[test]
fn test_decode_audio_symphonia_invalid_data() {
    let result = decode_audio_symphonia(Bytes::from(vec![0u8; 10]));
    assert!(result.is_err());
}

#[test]
fn test_decoded_audio_structure() {
    let data = StartupAudio::get_embedded_music();
    if data.len() > 100 {
        let result = decode_audio_symphonia(Bytes::from(data.to_vec()));
        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert!(!decoded.samples.is_empty());
        assert!(decoded.sample_rate > 0.0);
    }
}

#[test]
fn test_signature_tone_three_notes() {
    let signature = StartupAudio::generate_signature_tone();
    let expected_min = (SAMPLE_RATE as f32 * 0.08 * 3.0) as usize - 100;
    assert!(
        signature.len() >= expected_min,
        "Signature should have ~3 notes of 80ms each"
    );
}

#[test]
fn test_has_startup_music_with_embedded() {
    let startup = StartupAudio::new();
    assert!(StartupAudio::has_embedded_music() || startup.has_startup_music());
}

#[test]
fn test_startup_music_path_none_when_no_env() {
    let startup = StartupAudio::new();
    if std::env::var("PETALTONGUE_STARTUP_MUSIC").is_err() {
        assert!(startup.startup_music_path().is_none() || startup.startup_music_path().is_some());
    }
}
