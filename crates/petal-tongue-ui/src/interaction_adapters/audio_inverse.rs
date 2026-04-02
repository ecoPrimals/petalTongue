// SPDX-License-Identifier: AGPL-3.0-or-later
//! Audio inverse pipeline — resolves sonification back to data targets.
//!
//! When a blind user navigates through sonified data (frequency = value,
//! pan = position, rhythm = time), this pipeline maps the current audio
//! "cursor" position back to a [`DataObjectId`] so selection and inspection
//! work the same as in visual mode.
//!
//! The pipeline maintains a linearized sequence of data objects that
//! mirrors the sonification order. Keyboard navigation (Forward/Backward)
//! moves through this sequence; the current position maps to the data
//! object being sonified.

use petal_tongue_core::interaction::{
    DataObjectId, DataRow, InteractionContext, InteractionTarget, InversePipeline, OutputModality,
    PrimitiveId,
};
use petal_tongue_core::sensor::SensorEvent;

/// A data object in the sonification sequence.
#[derive(Debug, Clone)]
struct AudioTarget {
    data_id: DataObjectId,
    /// Index in the linearized sequence (for primitive resolution).
    sequence_index: usize,
}

/// Inverse pipeline for audio sonification output.
///
/// Resolves keyboard-driven navigation events to data targets by
/// maintaining a linearized traversal sequence that mirrors the
/// sonification order.
pub struct AudioInversePipeline {
    targets: Vec<AudioTarget>,
    current_index: usize,
}

impl AudioInversePipeline {
    /// Create an empty audio inverse pipeline.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            targets: Vec::new(),
            current_index: 0,
        }
    }

    /// Clear all targets (call when the scene changes).
    pub fn clear_targets(&mut self) {
        self.targets.clear();
        self.current_index = 0;
    }

    /// Register a data object in the sonification sequence.
    ///
    /// Objects should be registered in the order they are sonified
    /// (e.g. left-to-right for a time series, low-to-high for a bar chart).
    pub fn register_target(&mut self, data_id: DataObjectId) {
        let sequence_index = self.targets.len();
        self.targets.push(AudioTarget {
            data_id,
            sequence_index,
        });
    }

    /// Advance the cursor forward in the sequence.
    pub const fn advance(&mut self) {
        if !self.targets.is_empty() {
            self.current_index = (self.current_index + 1) % self.targets.len();
        }
    }

    /// Move the cursor backward in the sequence.
    pub const fn retreat(&mut self) {
        if !self.targets.is_empty() {
            self.current_index = match self.current_index.checked_sub(1) {
                Some(i) => i,
                None => self.targets.len() - 1,
            };
        }
    }

    /// Get the currently focused data object ID.
    #[must_use]
    pub fn current_data_id(&self) -> Option<&DataObjectId> {
        self.targets.get(self.current_index).map(|t| &t.data_id)
    }

    /// Get the total number of targets in the sequence.
    #[must_use]
    pub const fn target_count(&self) -> usize {
        self.targets.len()
    }

    /// Get the current cursor index.
    #[must_use]
    pub const fn current_index(&self) -> usize {
        self.current_index
    }
}

impl Default for AudioInversePipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl InversePipeline for AudioInversePipeline {
    fn modality(&self) -> OutputModality {
        OutputModality::Audio
    }

    fn resolve(
        &self,
        event: &SensorEvent,
        _context: &InteractionContext,
    ) -> Option<InteractionTarget> {
        match event {
            SensorEvent::KeyPress { .. } | SensorEvent::ButtonPress { .. } => self
                .targets
                .get(self.current_index)
                .map(|t| InteractionTarget::DataRow {
                    data_id: t.data_id.clone(),
                }),
            _ => None,
        }
    }

    fn nearest_primitive(&self, _target: &InteractionTarget) -> Option<PrimitiveId> {
        self.targets
            .get(self.current_index)
            .map(|t| t.sequence_index as PrimitiveId)
    }

    fn data_at(&self, _target: &InteractionTarget) -> Option<DataRow> {
        self.targets.get(self.current_index).map(|t| {
            let mut row = DataRow::new();
            row.insert("source".into(), serde_json::json!(t.data_id.source));
            row.insert("sequence_index".into(), serde_json::json!(t.sequence_index));
            row.insert("row_key".into(), t.data_id.row_key.clone());
            row
        })
    }

    fn resolve_to_data_id(&self, _target: &InteractionTarget) -> Option<DataObjectId> {
        self.current_data_id().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::interaction::InteractionContext;
    use petal_tongue_core::sensor::{Key, Modifiers, SensorEvent};
    use std::time::Instant;

    fn make_data_id(label: &str) -> DataObjectId {
        DataObjectId::new("test", serde_json::json!(label))
    }

    fn key_event() -> SensorEvent {
        SensorEvent::KeyPress {
            key: Key::Enter,
            modifiers: Modifiers::none(),
            timestamp: Instant::now(),
        }
    }

    #[test]
    fn empty_pipeline_resolves_none() {
        let pipeline = AudioInversePipeline::new();
        let ctx = InteractionContext::default_for_perspective(1);
        assert!(pipeline.resolve(&key_event(), &ctx).is_none());
        assert!(pipeline.current_data_id().is_none());
    }

    #[test]
    fn register_and_resolve() {
        let mut pipeline = AudioInversePipeline::new();
        pipeline.register_target(make_data_id("a"));
        pipeline.register_target(make_data_id("b"));
        pipeline.register_target(make_data_id("c"));

        assert_eq!(pipeline.target_count(), 3);
        assert_eq!(pipeline.current_index(), 0);

        let ctx = InteractionContext::default_for_perspective(1);
        let target = pipeline.resolve(&key_event(), &ctx);
        assert!(matches!(target, Some(InteractionTarget::DataRow { .. })));
    }

    #[test]
    fn advance_wraps_around() {
        let mut pipeline = AudioInversePipeline::new();
        pipeline.register_target(make_data_id("a"));
        pipeline.register_target(make_data_id("b"));

        pipeline.advance();
        assert_eq!(pipeline.current_index(), 1);
        pipeline.advance();
        assert_eq!(pipeline.current_index(), 0);
    }

    #[test]
    fn retreat_wraps_around() {
        let mut pipeline = AudioInversePipeline::new();
        pipeline.register_target(make_data_id("a"));
        pipeline.register_target(make_data_id("b"));

        pipeline.retreat();
        assert_eq!(pipeline.current_index(), 1);
        pipeline.retreat();
        assert_eq!(pipeline.current_index(), 0);
    }

    #[test]
    fn modality_is_audio() {
        let pipeline = AudioInversePipeline::new();
        assert_eq!(pipeline.modality(), OutputModality::Audio);
    }

    #[test]
    fn nearest_primitive_returns_sequence_index() {
        let mut pipeline = AudioInversePipeline::new();
        pipeline.register_target(make_data_id("x"));
        pipeline.register_target(make_data_id("y"));
        pipeline.advance();
        let prim = pipeline.nearest_primitive(&InteractionTarget::Nothing);
        assert_eq!(prim, Some(1));
    }

    #[test]
    fn data_at_returns_row() {
        let mut pipeline = AudioInversePipeline::new();
        pipeline.register_target(make_data_id("datum"));
        let row = pipeline.data_at(&InteractionTarget::Nothing);
        assert!(row.is_some());
        let row = row.unwrap();
        assert_eq!(row["source"], serde_json::json!("test"));
    }

    #[test]
    fn resolve_to_data_id_follows_cursor() {
        let mut pipeline = AudioInversePipeline::new();
        pipeline.register_target(make_data_id("first"));
        pipeline.register_target(make_data_id("second"));

        let id = pipeline.resolve_to_data_id(&InteractionTarget::Nothing);
        assert_eq!(id.as_ref().unwrap().row_key, serde_json::json!("first"));

        pipeline.advance();
        let id = pipeline.resolve_to_data_id(&InteractionTarget::Nothing);
        assert_eq!(id.as_ref().unwrap().row_key, serde_json::json!("second"));
    }

    #[test]
    fn clear_resets_state() {
        let mut pipeline = AudioInversePipeline::new();
        pipeline.register_target(make_data_id("a"));
        pipeline.advance();
        pipeline.clear_targets();
        assert_eq!(pipeline.target_count(), 0);
        assert_eq!(pipeline.current_index(), 0);
    }

    #[test]
    fn ignores_position_events() {
        let mut pipeline = AudioInversePipeline::new();
        pipeline.register_target(make_data_id("a"));
        let event = SensorEvent::Position {
            x: 0.0,
            y: 0.0,
            timestamp: Instant::now(),
        };
        let ctx = InteractionContext::default_for_perspective(1);
        assert!(pipeline.resolve(&event, &ctx).is_none());
    }
}
