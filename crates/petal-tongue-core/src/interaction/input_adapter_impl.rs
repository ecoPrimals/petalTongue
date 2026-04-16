// SPDX-License-Identifier: AGPL-3.0-or-later
//! Enum dispatch for [`InputAdapter`] (replaces `Box<dyn InputAdapter>`).

use crate::interaction::adapter::{
    InputAdapter, InputModality, InteractionCapability, InteractionContext,
};
use crate::interaction::adapters::{
    AgentInputAdapter, KeyboardAdapter, PointerAdapter, SwitchInputAdapter,
};
use crate::interaction::intent::InteractionIntent;
use crate::interaction::result::InteractionResult;
use crate::interaction::target::InteractionTarget;
use crate::sensor::SensorEvent;

#[cfg(test)]
use crate::interaction::mock_adapters::{MockInputAdapter, QueuedInputAdapter};

/// Concrete input adapters registered on [`InteractionEngine`](crate::interaction::InteractionEngine).
pub enum InputAdapterImpl {
    /// Pointer / touch adapter.
    Pointer(PointerAdapter),
    /// Keyboard adapter.
    Keyboard(KeyboardAdapter),
    /// Switch / discrete control adapter.
    Switch(SwitchInputAdapter),
    /// Agent-driven (JSON) adapter.
    Agent(AgentInputAdapter),
    /// Test double (unit tests).
    #[cfg(test)]
    Mock(MockInputAdapter),
    /// Queued events (unit tests).
    #[cfg(test)]
    Queued(QueuedInputAdapter),
}

impl InputAdapter for InputAdapterImpl {
    fn name(&self) -> &'static str {
        match self {
            Self::Pointer(a) => a.name(),
            Self::Keyboard(a) => a.name(),
            Self::Switch(a) => a.name(),
            Self::Agent(a) => a.name(),
            #[cfg(test)]
            Self::Mock(a) => a.name(),
            #[cfg(test)]
            Self::Queued(a) => a.name(),
        }
    }

    fn modality(&self) -> InputModality {
        match self {
            Self::Pointer(a) => a.modality(),
            Self::Keyboard(a) => a.modality(),
            Self::Switch(a) => a.modality(),
            Self::Agent(a) => a.modality(),
            #[cfg(test)]
            Self::Mock(a) => a.modality(),
            #[cfg(test)]
            Self::Queued(a) => a.modality(),
        }
    }

    fn capabilities(&self) -> &[InteractionCapability] {
        match self {
            Self::Pointer(a) => a.capabilities(),
            Self::Keyboard(a) => a.capabilities(),
            Self::Switch(a) => a.capabilities(),
            Self::Agent(a) => a.capabilities(),
            #[cfg(test)]
            Self::Mock(a) => a.capabilities(),
            #[cfg(test)]
            Self::Queued(a) => a.capabilities(),
        }
    }

    fn translate(
        &self,
        event: &SensorEvent,
        context: &InteractionContext,
    ) -> Option<InteractionIntent> {
        match self {
            Self::Pointer(a) => a.translate(event, context),
            Self::Keyboard(a) => a.translate(event, context),
            Self::Switch(a) => a.translate(event, context),
            Self::Agent(a) => a.translate(event, context),
            #[cfg(test)]
            Self::Mock(a) => a.translate(event, context),
            #[cfg(test)]
            Self::Queued(a) => a.translate(event, context),
        }
    }

    fn active_target(&self, context: &InteractionContext) -> Option<InteractionTarget> {
        match self {
            Self::Pointer(a) => a.active_target(context),
            Self::Keyboard(a) => a.active_target(context),
            Self::Switch(a) => a.active_target(context),
            Self::Agent(a) => a.active_target(context),
            #[cfg(test)]
            Self::Mock(a) => a.active_target(context),
            #[cfg(test)]
            Self::Queued(a) => a.active_target(context),
        }
    }

    fn feedback(&mut self, result: &InteractionResult) {
        match self {
            Self::Pointer(a) => a.feedback(result),
            Self::Keyboard(a) => a.feedback(result),
            Self::Switch(a) => a.feedback(result),
            Self::Agent(a) => a.feedback(result),
            #[cfg(test)]
            Self::Mock(a) => a.feedback(result),
            #[cfg(test)]
            Self::Queued(a) => a.feedback(result),
        }
    }
}
