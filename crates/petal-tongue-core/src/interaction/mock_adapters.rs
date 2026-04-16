// SPDX-License-Identifier: AGPL-3.0-or-later
//! Test-only input adapters for the interaction engine.

use crate::interaction::adapter::{
    InputAdapter, InputModality, InteractionCapability, InteractionContext,
};
use crate::interaction::intent::InteractionIntent;
use crate::interaction::result::InteractionResult;
use crate::interaction::target::InteractionTarget;
use crate::sensor::SensorEvent;

/// Returns a fixed intent (or none) — for engine unit tests.
pub struct MockInputAdapter {
    pub translate_returns: Option<InteractionIntent>,
}

impl InputAdapter for MockInputAdapter {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn modality(&self) -> InputModality {
        InputModality::PointerMouse
    }

    fn capabilities(&self) -> &[InteractionCapability] {
        &[InteractionCapability::PointSelect]
    }

    fn translate(
        &self,
        _event: &SensorEvent,
        _context: &InteractionContext,
    ) -> Option<InteractionIntent> {
        self.translate_returns.clone()
    }

    fn active_target(&self, _context: &InteractionContext) -> Option<InteractionTarget> {
        None
    }

    fn feedback(&mut self, _result: &InteractionResult) {}
}

/// Dequeues intents in order — for engine unit tests.
pub struct QueuedInputAdapter {
    pub intents: std::sync::Mutex<Vec<InteractionIntent>>,
}

impl InputAdapter for QueuedInputAdapter {
    fn name(&self) -> &'static str {
        "queued"
    }

    fn modality(&self) -> InputModality {
        InputModality::PointerMouse
    }

    fn capabilities(&self) -> &[InteractionCapability] {
        &[InteractionCapability::PointSelect]
    }

    fn translate(
        &self,
        _event: &SensorEvent,
        _context: &InteractionContext,
    ) -> Option<InteractionIntent> {
        let mut intents = self.intents.lock().expect("lock");
        if intents.is_empty() {
            None
        } else {
            Some(intents.remove(0))
        }
    }

    fn active_target(&self, _context: &InteractionContext) -> Option<InteractionTarget> {
        None
    }

    fn feedback(&mut self, _result: &InteractionResult) {}
}
