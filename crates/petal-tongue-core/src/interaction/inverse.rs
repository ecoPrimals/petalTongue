// SPDX-License-Identifier: AGPL-3.0-only
//! Generalized inverse pipeline trait.
//!
//! The forward pipeline maps data to modality output (pixels, tones, cells).
//! The inverse pipeline maps interactions BACK to data. Each output modality
//! has its own inverse path, but all converge to the same
//! [`DataObjectId`](super::target::DataObjectId).

use crate::sensor::SensorEvent;

use super::adapter::InteractionContext;
use super::perspective::OutputModality;
use super::target::{DataObjectId, DataRow, InteractionTarget, PrimitiveId};

/// Resolves modality-specific interaction positions to data-space targets.
///
/// Each [`ModalityCompiler`] in the Grammar of Graphics pipeline produces
/// a corresponding `InversePipeline`. The compiler knows the forward mapping;
/// the inverse is its mirror.
///
/// # Modality-Specific Resolution
///
/// - **Visual**: pixel coords -> viewport normalize -> inverse coord system -> data values
/// - **Audio**: time offset -> tone mapping -> inverse sonification -> data row
/// - **TUI**: cursor (row, col) -> cell content -> inverse character mapping -> data value
/// - **Voice**: parsed command -> entity resolution -> data object
pub trait InversePipeline: Send + Sync {
    /// Which output modality this pipeline inverts.
    fn modality(&self) -> OutputModality;

    /// Resolve a sensor event to an interaction target.
    ///
    /// This is the primary entry point. Given a device event and the current
    /// render state, determine what data object (if any) the user is
    /// pointing at / interacting with.
    fn resolve(
        &self,
        event: &SensorEvent,
        context: &InteractionContext,
    ) -> Option<InteractionTarget>;

    /// Find the nearest rendered primitive to a target.
    ///
    /// Used for snapping: when the user's input is between two data points,
    /// find the closest one.
    fn nearest_primitive(&self, target: &InteractionTarget) -> Option<PrimitiveId>;

    /// Resolve an interaction target to actual data values.
    ///
    /// Given a target (which may still be in screen/modality space),
    /// return the data row it corresponds to.
    fn data_at(&self, target: &InteractionTarget) -> Option<DataRow>;

    /// Resolve an interaction target to a stable data object ID.
    ///
    /// This is the final step: from any modality-specific target, produce
    /// the perspective-invariant `DataObjectId` that can be shared across
    /// modalities and users.
    fn resolve_to_data_id(&self, target: &InteractionTarget) -> Option<DataObjectId>;
}

/// A no-op inverse pipeline for modalities that don't support interaction.
///
/// Used for export-only modalities (SVG, PNG, headless) that produce output
/// but don't accept input back.
pub struct NoOpInversePipeline {
    modality: OutputModality,
}

impl NoOpInversePipeline {
    /// Create a no-op pipeline for a given modality.
    #[must_use]
    pub fn new(modality: OutputModality) -> Self {
        Self { modality }
    }
}

impl InversePipeline for NoOpInversePipeline {
    fn modality(&self) -> OutputModality {
        self.modality
    }

    fn resolve(
        &self,
        _event: &SensorEvent,
        _context: &InteractionContext,
    ) -> Option<InteractionTarget> {
        None
    }

    fn nearest_primitive(&self, _target: &InteractionTarget) -> Option<PrimitiveId> {
        None
    }

    fn data_at(&self, _target: &InteractionTarget) -> Option<DataRow> {
        None
    }

    fn resolve_to_data_id(&self, _target: &InteractionTarget) -> Option<DataObjectId> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn noop_pipeline_returns_none() {
        let pipeline = NoOpInversePipeline::new(OutputModality::Svg);
        assert_eq!(pipeline.modality(), OutputModality::Svg);

        let event = SensorEvent::Click {
            x: 100.0,
            y: 200.0,
            button: crate::sensor::MouseButton::Left,
            timestamp: Instant::now(),
        };
        let ctx = InteractionContext::default_for_perspective(1);
        assert!(pipeline.resolve(&event, &ctx).is_none());
        assert!(
            pipeline
                .nearest_primitive(&InteractionTarget::Nothing)
                .is_none()
        );
        assert!(pipeline.data_at(&InteractionTarget::Nothing).is_none());
        assert!(
            pipeline
                .resolve_to_data_id(&InteractionTarget::Nothing)
                .is_none()
        );
    }
}
