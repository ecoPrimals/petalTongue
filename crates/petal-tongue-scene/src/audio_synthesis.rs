// SPDX-License-Identifier: AGPL-3.0-or-later
//! Deterministic audio synthesis and per-sample provenance.

use std::f64::consts::PI;

use crate::modality::AudioParam;

/// Pure Rust sample generator. Same params = same samples, always (deterministic).
#[derive(Debug, Clone)]
pub struct AudioSynthesizer {
    pub sample_rate: u32,
}

impl AudioSynthesizer {
    /// Create a new synthesizer (default 44100 Hz).
    #[must_use]
    pub const fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    /// Generate PCM samples for a single `AudioParam`.
    /// Duration determines sample count; frequency → sine wave; pan stored but mono output.
    #[must_use]
    pub fn synthesize(&self, param: &AudioParam) -> Vec<f32> {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "duration * sample_rate is non-negative; truncation acceptable"
        )]
        let n = (param.duration_secs * f64::from(self.sample_rate)) as usize;
        #[expect(clippy::cast_precision_loss, reason = "time for phase: f64 sufficient")]
        #[expect(
            clippy::cast_possible_truncation,
            reason = "PCM output clamped by amplitude"
        )]
        (0..n)
            .map(|i| {
                let t = i as f64 / f64::from(self.sample_rate);
                ((2.0 * PI * param.frequency * t).sin() * param.amplitude) as f32
            })
            .collect()
    }

    /// Concatenate all synthesized samples from multiple params.
    #[must_use]
    pub fn synthesize_all(&self, params: &[AudioParam]) -> Vec<f32> {
        params.iter().flat_map(|p| self.synthesize(p)).collect()
    }
}

impl Default for AudioSynthesizer {
    fn default() -> Self {
        Self::new(44100)
    }
}

/// Per-sample provenance for a synthesized segment.
#[derive(Debug, Clone)]
pub struct AudioProvenance {
    pub param_index: usize,
    pub data_id: Option<String>,
    pub frequency: f64,
    pub amplitude: f64,
}

/// Maps sample index ranges to provenance.
#[derive(Debug, Clone, Default)]
pub struct AudioProvenanceMap {
    entries: Vec<(usize, usize, AudioProvenance)>,
}

impl AudioProvenanceMap {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn register(&mut self, start: usize, end: usize, provenance: AudioProvenance) {
        self.entries.push((start, end, provenance));
    }

    /// Returns provenance for a sample index if it falls within a registered range.
    #[must_use]
    pub fn query(&self, sample_index: usize) -> Option<&AudioProvenance> {
        self.entries
            .iter()
            .find(|(s, e, _)| *s <= sample_index && sample_index < *e)
            .map(|(_, _, p)| p)
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Generate samples and build provenance map simultaneously.
#[must_use]
pub fn synthesize_with_provenance(
    synth: &AudioSynthesizer,
    params: &[AudioParam],
    data_ids: &[Option<String>],
) -> (Vec<f32>, AudioProvenanceMap) {
    let mut samples = Vec::new();
    let mut map = AudioProvenanceMap::new();
    let mut offset = 0usize;

    for (i, param) in params.iter().enumerate() {
        let seg = synth.synthesize(param);
        let start = offset;
        let end = offset + seg.len();
        offset = end;

        let data_id = data_ids.get(i).and_then(Clone::clone);
        map.register(
            start,
            end,
            AudioProvenance {
                param_index: i,
                data_id,
                frequency: param.frequency,
                amplitude: param.amplitude,
            },
        );
        samples.extend(seg);
    }

    (samples, map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modality::AudioParam;

    fn param(freq: f64, amp: f64, dur: f64) -> AudioParam {
        AudioParam {
            frequency: freq,
            amplitude: amp,
            pan: 0.0,
            duration_secs: dur,
        }
    }

    #[test]
    fn determinism_same_input_same_output() {
        let synth = AudioSynthesizer::new(44100);
        let p = param(440.0, 0.5, 0.1);
        let a = synth.synthesize(&p);
        let b = synth.synthesize(&p);
        assert_eq!(a, b);
    }

    #[test]
    fn correct_sample_count() {
        let synth = AudioSynthesizer::new(44100);
        let p = param(440.0, 0.5, 0.1);
        let samples = synth.synthesize(&p);
        #[expect(
            clippy::cast_sign_loss,
            reason = "test: duration * sample_rate is non-negative"
        )]
        let expected = (0.1 * 44100.0) as usize;
        assert_eq!(samples.len(), expected);
    }

    #[test]
    fn zero_amplitude_is_silence() {
        let synth = AudioSynthesizer::new(44100);
        let p = param(440.0, 0.0, 0.1);
        let samples = synth.synthesize(&p);
        assert!(samples.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn provenance_map_register_and_query() {
        let mut map = AudioProvenanceMap::new();
        map.register(
            0,
            100,
            AudioProvenance {
                param_index: 0,
                data_id: Some("d1".to_string()),
                frequency: 440.0,
                amplitude: 0.5,
            },
        );
        map.register(
            100,
            200,
            AudioProvenance {
                param_index: 1,
                data_id: None,
                frequency: 880.0,
                amplitude: 0.3,
            },
        );

        let p0 = map.query(50).unwrap();
        assert_eq!(p0.param_index, 0);
        assert_eq!(p0.frequency, 440.0);

        let p1 = map.query(150).unwrap();
        assert_eq!(p1.param_index, 1);
        assert_eq!(p1.frequency, 880.0);

        assert!(map.query(200).is_none());
        assert!(map.query(250).is_none());
    }

    #[test]
    fn provenance_map_len_and_is_empty() {
        let mut map = AudioProvenanceMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        map.register(
            0,
            10,
            AudioProvenance {
                param_index: 0,
                data_id: None,
                frequency: 1.0,
                amplitude: 1.0,
            },
        );
        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);
    }
}
