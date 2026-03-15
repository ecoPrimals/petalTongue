// SPDX-License-Identifier: AGPL-3.0-only
//! State synchronization coordinator.

use crate::adaptive_rendering::DeviceType;
use crate::dynamic_schema::DynamicValue;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;

use super::persistence::StatePersistence;
use super::types::DeviceState;

/// State synchronization coordinator
pub struct StateSync {
    /// Local persistence
    persistence: Box<dyn StatePersistence>,

    /// Current device state (Arc for zero-copy sharing)
    current_state: Option<Arc<DeviceState>>,
}

impl StateSync {
    /// Create a new state sync with local persistence
    pub fn new() -> Result<Self> {
        Ok(Self {
            persistence: Box::new(super::persistence::LocalStatePersistence::new()?),
            current_state: None,
        })
    }

    /// Create with custom persistence (for testing)
    #[cfg(test)]
    pub fn with_persistence(persistence: Box<dyn StatePersistence>) -> Self {
        Self {
            persistence,
            current_state: None,
        }
    }

    /// Initialize state for this device
    pub fn init(
        &mut self,
        device_id: impl Into<String>,
        device_type: DeviceType,
    ) -> Result<Arc<DeviceState>> {
        let device_id = device_id.into();
        if let Some(mut state) = self.persistence.load(&device_id)? {
            tracing::info!("📂 Loaded existing state for device {}", device_id);
            state.device_type = device_type;
            state.last_updated = Utc::now();
            let state = Arc::new(state);
            self.current_state = Some(Arc::clone(&state));
            return Ok(state);
        }

        tracing::info!("🆕 Creating new state for device {}", device_id);
        let state = Arc::new(DeviceState::new(device_id, device_type));
        self.current_state = Some(Arc::clone(&state));
        Ok(state)
    }

    /// Get current state (reference into shared Arc)
    #[must_use]
    pub fn current(&self) -> Option<&DeviceState> {
        self.current_state.as_deref()
    }

    /// Update current state
    pub fn update(&mut self, state: DeviceState) -> Result<()> {
        self.persistence.save(&state)?;
        self.current_state = Some(Arc::new(state));
        Ok(())
    }

    /// Set UI state value
    pub fn set_ui_state(&mut self, key: impl Into<String>, value: DynamicValue) -> Result<()> {
        if let Some(arc) = &self.current_state {
            let mut state = (**arc).clone();
            state.set_ui_state(key, value);
            self.persistence.save(&state)?;
            self.current_state = Some(Arc::new(state));
        }
        Ok(())
    }

    /// Get UI state value
    #[must_use]
    pub fn get_ui_state(&self, key: &str) -> Option<&DynamicValue> {
        self.current_state.as_ref()?.get_ui_state(key)
    }
}
