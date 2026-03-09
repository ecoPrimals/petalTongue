// SPDX-License-Identifier: AGPL-3.0-only
//! Narrative entropy capture (storytelling, typing)
//!
//! Captures keystroke dynamics, typing rhythm, and content uniqueness.

use crate::quality::{shannon_entropy, timing_entropy, weighted_quality};
use crate::types::*;
use std::time::{Duration, Instant};

/// Compute Shannon entropy over character frequencies in the input text.
///
/// H = -Σ p(x) * log2(p(x)) where p(x) is the probability of each character.
/// Returns raw bits of entropy (not normalized). Empty input returns 0.0.
///
/// # Arguments
/// * `text` - Input text to analyze
///
/// # Returns
/// Shannon entropy in bits, or 0.0 for empty input
pub fn compute_text_entropy(text: &str) -> f64 {
    if text.is_empty() {
        return 0.0;
    }

    let chars: Vec<char> = text.chars().collect();
    let total = chars.len() as f64;

    let mut counts: std::collections::HashMap<char, usize> = std::collections::HashMap::new();
    for c in &chars {
        *counts.entry(*c).or_insert(0) += 1;
    }

    let entropy: f64 = counts
        .values()
        .map(|&count| {
            let p = count as f64 / total;
            -p * p.log2()
        })
        .sum();

    entropy
}

/// Quantify narrative complexity from vocabulary diversity, average sentence length,
/// and punctuation density.
///
/// # Arguments
/// * `text` - Input text to analyze
///
/// # Returns
/// Complexity score [0.0-1.0], or 0.0 for empty input
pub fn narrative_complexity(text: &str) -> f64 {
    if text.is_empty() {
        return 0.0;
    }

    let words: Vec<&str> = text.split_whitespace().filter(|s| !s.is_empty()).collect();
    let total_words = words.len();
    if total_words == 0 {
        return 0.0;
    }

    // 1. Vocabulary diversity: unique words / total words
    let unique_words: std::collections::HashSet<&str> = words.iter().copied().collect();
    let vocab_diversity = unique_words.len() as f64 / total_words as f64;

    // 2. Average sentence length (words per sentence)
    let sentences: Vec<&str> = text
        .split(|c: char| ['.', '!', '?'].contains(&c))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let num_sentences = sentences.len().max(1);
    let avg_sentence_len = total_words as f64 / num_sentences as f64;
    // Normalize: typical sentence 5-20 words → 0.2-0.8
    let sentence_component = (avg_sentence_len / 25.0).min(1.0);

    // 3. Punctuation density
    let punct_count = text.chars().filter(|c| c.is_ascii_punctuation()).count();
    let punct_density = punct_count as f64 / text.len() as f64;
    let punct_component = (punct_density * 10.0).min(1.0);

    (vocab_diversity * 0.5 + sentence_component * 0.25 + punct_component * 0.25).clamp(0.0, 1.0)
}

/// Narrative entropy capturer
pub struct NarrativeEntropyCapture {
    text: String,
    keystroke_timings: Vec<Duration>,
    backspace_events: Vec<BackspaceEvent>,
    start_time: Option<Instant>,
    last_keystroke: Option<Instant>,
}

impl NarrativeEntropyCapture {
    /// Create a new narrative entropy capturer
    pub fn new() -> Self {
        Self {
            text: String::new(),
            keystroke_timings: Vec::new(),
            backspace_events: Vec::new(),
            start_time: None,
            last_keystroke: None,
        }
    }

    /// Start capturing
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.last_keystroke = Some(Instant::now());
    }

    /// Add a character
    pub fn add_char(&mut self, c: char) {
        self.text.push(c);

        if let Some(last) = self.last_keystroke {
            let now = Instant::now();
            self.keystroke_timings.push(now.duration_since(last));
            self.last_keystroke = Some(now);
        }
    }

    /// Record backspace event
    pub fn add_backspace(&mut self) {
        if let (Some(start), Some(c)) = (self.start_time, self.text.pop()) {
            let timestamp = Instant::now().duration_since(start);
            self.backspace_events.push(BackspaceEvent {
                timestamp,
                position: self.text.len(),
                deleted_char: c,
            });
        }
    }

    /// Assess current quality
    pub fn assess_quality(&self) -> NarrativeQualityMetrics {
        if self.text.is_empty() {
            return NarrativeQualityMetrics {
                keystroke_entropy: 0.0,
                pause_entropy: 0.0,
                correction_entropy: 0.0,
                content_entropy: 0.0,
                overall_quality: 0.0,
            };
        }

        let keystroke_entropy = timing_entropy(&self.keystroke_timings);
        let pause_entropy = self.calculate_pause_entropy();
        let correction_entropy = self.calculate_correction_entropy();
        let content_entropy = self.calculate_content_entropy();

        let overall_quality = weighted_quality(&[
            (keystroke_entropy, 0.3),
            (pause_entropy, 0.2),
            (correction_entropy, 0.2),
            (content_entropy, 0.3),
        ]);

        NarrativeQualityMetrics {
            keystroke_entropy,
            pause_entropy,
            correction_entropy,
            content_entropy,
            overall_quality,
        }
    }

    fn calculate_pause_entropy(&self) -> f64 {
        let long_pauses: Vec<Duration> = self
            .keystroke_timings
            .iter()
            .filter(|d| d.as_millis() > 500)
            .copied()
            .collect();
        if long_pauses.len() < 2 {
            return if long_pauses.is_empty() { 0.0 } else { 0.5 };
        }
        timing_entropy(&long_pauses)
    }

    fn calculate_correction_entropy(&self) -> f64 {
        if self.backspace_events.is_empty() {
            return 0.0;
        }
        let positions: Vec<usize> = self.backspace_events.iter().map(|e| e.position).collect();
        let buckets = crate::quality::create_histogram_buckets(
            &positions.iter().map(|&p| p as f64).collect::<Vec<_>>(),
            10,
        );
        if buckets.is_empty() {
            return 0.5;
        }
        shannon_entropy(&buckets)
    }

    fn calculate_content_entropy(&self) -> f64 {
        if self.text.is_empty() {
            return 0.0;
        }
        let chars: Vec<char> = self.text.chars().collect();
        if chars.len() < 2 {
            return 0.0;
        }
        let num_unique = chars.iter().collect::<std::collections::HashSet<_>>().len();
        if num_unique <= 1 {
            return 0.0;
        }
        let raw_entropy = compute_text_entropy(&self.text);
        let max_entropy = (num_unique as f64).log2();
        (raw_entropy / max_entropy).min(1.0)
    }

    /// Finalize and create entropy data
    pub fn finalize(self) -> anyhow::Result<NarrativeEntropyData> {
        let quality_metrics = self.assess_quality();

        // Extract pause durations (long gaps in keystroke timings)
        let pause_durations: Vec<Duration> = self
            .keystroke_timings
            .iter()
            .filter(|d| d.as_millis() > 500) // > 500ms = pause
            .copied()
            .collect();

        Ok(NarrativeEntropyData {
            text: self.text,
            keystroke_timings: self.keystroke_timings,
            backspace_events: self.backspace_events,
            pause_durations,
            quality_metrics,
        })
    }
}

impl Default for NarrativeEntropyCapture {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_narrative_capture_creation() {
        let capture = NarrativeEntropyCapture::new();
        assert!(capture.text.is_empty());
    }

    #[test]
    fn test_narrative_add_char() {
        let mut capture = NarrativeEntropyCapture::new();
        capture.start();
        capture.add_char('H');
        capture.add_char('i');
        assert_eq!(capture.text, "Hi");
    }

    #[test]
    fn test_narrative_quality_empty() {
        let capture = NarrativeEntropyCapture::new();
        let quality = capture.assess_quality();
        assert_eq!(quality.overall_quality, 0.0);
    }

    #[test]
    fn test_compute_text_entropy_empty() {
        assert_eq!(compute_text_entropy(""), 0.0);
    }

    #[test]
    fn test_compute_text_entropy_single_char() {
        assert_eq!(compute_text_entropy("a"), 0.0);
    }

    #[test]
    fn test_compute_text_entropy_uniform() {
        let text = "abcd";
        let entropy = compute_text_entropy(text);
        assert!((entropy - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_compute_text_entropy_repeated() {
        let text = "aaaa";
        assert_eq!(compute_text_entropy(text), 0.0);
    }

    #[test]
    fn test_narrative_complexity_empty() {
        assert_eq!(narrative_complexity(""), 0.0);
    }

    #[test]
    fn test_narrative_complexity_simple() {
        let text = "Hello world.";
        let complexity = narrative_complexity(text);
        assert!(complexity > 0.0 && complexity <= 1.0);
    }

    #[test]
    fn test_narrative_complexity_diverse() {
        let text =
            "The quick brown fox jumps over the lazy dog. What a wonderful day! How are you?";
        let complexity = narrative_complexity(text);
        assert!(complexity > 0.0 && complexity <= 1.0);
    }
}
