// SPDX-License-Identifier: AGPL-3.0-only
//! Deterministic haptic sequence from commands.

use crate::modality::{HapticCommand, HapticPattern};

/// A single timed haptic event.
#[derive(Debug, Clone)]
pub struct HapticPulse {
    pub start_secs: f64,
    pub duration_secs: f64,
    pub intensity: f64,
    pub position: [f64; 2],
    pub pattern: HapticPattern,
    pub data_id: Option<String>,
}

/// Ordered timeline of pulses.
#[derive(Debug, Clone, Default)]
pub struct HapticSequence {
    pulses: Vec<HapticPulse>,
    total_duration: f64,
}

impl HapticSequence {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pulses: Vec::new(),
            total_duration: 0.0,
        }
    }

    /// Convert `HapticCommands` to a time-ordered sequence.
    /// Commands are placed sequentially: first at 0, next at end of previous, etc.
    #[must_use]
    pub fn from_commands(commands: &[HapticCommand], data_ids: &[Option<String>]) -> Self {
        let mut pulses = Vec::with_capacity(commands.len());
        let mut t = 0.0;

        for (i, cmd) in commands.iter().enumerate() {
            let data_id = data_ids.get(i).and_then(Clone::clone);
            pulses.push(HapticPulse {
                start_secs: t,
                duration_secs: cmd.duration_secs,
                intensity: cmd.intensity,
                position: cmd.position,
                pattern: cmd.pattern,
                data_id,
            });
            t += cmd.duration_secs;
        }

        Self {
            pulses,
            total_duration: t,
        }
    }

    /// Returns pulses active at the given time (start <= time < start + duration).
    #[must_use]
    pub fn query_at(&self, time_secs: f64) -> Vec<&HapticPulse> {
        self.pulses
            .iter()
            .filter(|p| time_secs >= p.start_secs && time_secs < p.start_secs + p.duration_secs)
            .collect()
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.pulses.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.pulses.is_empty()
    }

    #[must_use]
    pub const fn total_duration(&self) -> f64 {
        self.total_duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modality::HapticPattern;

    fn cmd(intensity: f64, dur: f64, pos: [f64; 2], pattern: HapticPattern) -> HapticCommand {
        HapticCommand {
            intensity,
            duration_secs: dur,
            position: pos,
            pattern,
        }
    }

    #[test]
    fn from_commands_produces_correct_pulses() {
        let commands = vec![
            cmd(0.5, 0.1, [0.2, 0.3], HapticPattern::Pulse),
            cmd(0.8, 0.2, [0.5, 0.5], HapticPattern::Sustained),
        ];
        let data_ids = vec![Some("d1".to_string()), None];
        let seq = HapticSequence::from_commands(&commands, &data_ids);

        assert_eq!(seq.len(), 2);
        assert!((seq.total_duration() - 0.3).abs() < 1e-10);

        let p0 = seq.query_at(0.05);
        assert_eq!(p0.len(), 1);
        assert_eq!(p0[0].start_secs, 0.0);
        assert_eq!(p0[0].duration_secs, 0.1);
        assert_eq!(p0[0].intensity, 0.5);
        assert_eq!(p0[0].data_id.as_deref(), Some("d1"));
        assert_eq!(p0[0].pattern, HapticPattern::Pulse);

        let p1 = seq.query_at(0.15);
        assert_eq!(p1.len(), 1);
        assert_eq!(p1[0].start_secs, 0.1);
        assert_eq!(p1[0].duration_secs, 0.2);
        assert_eq!(p1[0].intensity, 0.8);
        assert_eq!(p1[0].pattern, HapticPattern::Sustained);
    }

    #[test]
    fn query_at_returns_active_pulses() {
        let commands = vec![
            cmd(0.5, 0.1, [0.0, 0.0], HapticPattern::Pulse),
            cmd(0.8, 0.2, [0.5, 0.5], HapticPattern::Sustained),
        ];
        let seq = HapticSequence::from_commands(&commands, &[]);

        let at_0 = seq.query_at(0.0);
        assert_eq!(at_0.len(), 1);
        assert_eq!(at_0[0].pattern, HapticPattern::Pulse);

        let at_05 = seq.query_at(0.05);
        assert_eq!(at_05.len(), 1);

        let at_1 = seq.query_at(0.1);
        assert_eq!(at_1.len(), 1);
        assert_eq!(at_1[0].pattern, HapticPattern::Sustained);

        let at_15 = seq.query_at(0.15);
        assert_eq!(at_15.len(), 1);

        let at_31 = seq.query_at(0.31);
        assert!(at_31.is_empty());

        let at_neg = seq.query_at(-0.1);
        assert!(at_neg.is_empty());
    }

    #[test]
    fn empty_sequence() {
        let seq = HapticSequence::from_commands(&[], &[]);
        assert!(seq.is_empty());
        assert_eq!(seq.len(), 0);
        assert_eq!(seq.total_duration(), 0.0);
        assert!(seq.query_at(0.0).is_empty());
    }
}
