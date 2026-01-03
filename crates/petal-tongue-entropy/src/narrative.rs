//! Narrative entropy capture (storytelling, typing)
//!
//! Captures keystroke dynamics, typing rhythm, and content uniqueness.

use crate::quality::{shannon_entropy, timing_entropy, weighted_quality};
use crate::types::*;
use std::time::{Duration, Instant};

/// Narrative entropy capturer (stub for Phase 4)
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
        // Stub: Analyze long pauses (thinking time)
        0.7 // Placeholder
    }

    fn calculate_correction_entropy(&self) -> f64 {
        // Stub: Analyze backspace patterns
        if self.backspace_events.is_empty() {
            0.5
        } else {
            0.7
        }
    }

    fn calculate_content_entropy(&self) -> f64 {
        // Stub: Calculate character bigram entropy
        let chars: Vec<char> = self.text.chars().collect();
        if chars.len() < 2 {
            return 0.0;
        }

        let bigrams: Vec<String> = chars
            .windows(2)
            .map(|w| format!("{}{}", w[0], w[1]))
            .collect();

        shannon_entropy(&bigrams)
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
}
