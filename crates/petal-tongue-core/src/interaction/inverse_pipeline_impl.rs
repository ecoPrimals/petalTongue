// SPDX-License-Identifier: AGPL-3.0-or-later
//! Enum dispatch for [`InversePipeline`] (replaces `Box<dyn InversePipeline>`).

use crate::interaction::adapter::InteractionContext;
use crate::interaction::adapters::{AudioInversePipeline, VisualInversePipeline};
use crate::interaction::inverse::{InversePipeline, NoOpInversePipeline};
use crate::interaction::perspective::OutputModality;
use crate::interaction::target::{DataObjectId, DataRow, InteractionTarget, PrimitiveId};
use crate::sensor::SensorEvent;

/// Concrete inverse pipelines registered on [`InteractionEngine`](crate::interaction::InteractionEngine).
pub enum InversePipelineImpl {
    /// Visual output modality.
    Visual(VisualInversePipeline),
    /// Audio output modality.
    Audio(AudioInversePipeline),
    /// No-op pipeline (ignored modality).
    NoOp(NoOpInversePipeline),
}

impl InversePipeline for InversePipelineImpl {
    fn modality(&self) -> OutputModality {
        match self {
            Self::Visual(p) => p.modality(),
            Self::Audio(p) => p.modality(),
            Self::NoOp(p) => p.modality(),
        }
    }

    fn resolve(
        &self,
        event: &SensorEvent,
        context: &InteractionContext,
    ) -> Option<InteractionTarget> {
        match self {
            Self::Visual(p) => p.resolve(event, context),
            Self::Audio(p) => p.resolve(event, context),
            Self::NoOp(p) => p.resolve(event, context),
        }
    }

    fn nearest_primitive(&self, target: &InteractionTarget) -> Option<PrimitiveId> {
        match self {
            Self::Visual(p) => p.nearest_primitive(target),
            Self::Audio(p) => p.nearest_primitive(target),
            Self::NoOp(p) => p.nearest_primitive(target),
        }
    }

    fn data_at(&self, target: &InteractionTarget) -> Option<DataRow> {
        match self {
            Self::Visual(p) => p.data_at(target),
            Self::Audio(p) => p.data_at(target),
            Self::NoOp(p) => p.data_at(target),
        }
    }

    fn resolve_to_data_id(&self, target: &InteractionTarget) -> Option<DataObjectId> {
        match self {
            Self::Visual(p) => p.resolve_to_data_id(target),
            Self::Audio(p) => p.resolve_to_data_id(target),
            Self::NoOp(p) => p.resolve_to_data_id(target),
        }
    }
}
