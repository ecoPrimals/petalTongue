// SPDX-License-Identifier: AGPL-3.0-or-later
//! State synchronization coordinator.

use crate::adaptive_rendering::DeviceType;
use crate::dynamic_schema::DynamicValue;
use crate::error::Result;
use chrono::Utc;
use std::sync::Arc;

use super::persistence::{StatePersistence, StatePersistenceImpl};
use super::types::DeviceState;

/// State synchronization coordinator
pub struct StateSync {
    /// Local persistence
    persistence: StatePersistenceImpl,

    /// Current device state (Arc for zero-copy sharing)
    current_state: Option<Arc<DeviceState>>,
}

impl StateSync {
    /// Create a new state sync with local persistence
    ///
    /// # Errors
    ///
    /// Returns an error if the config directory cannot be determined or the
    /// state directory cannot be created.
    pub fn new() -> Result<Self> {
        Ok(Self {
            persistence: StatePersistenceImpl::Local(
                super::persistence::LocalStatePersistence::new()?,
            ),
            current_state: None,
        })
    }

    /// Create with custom persistence (for testing)
    #[cfg(test)]
    #[must_use]
    pub const fn with_persistence(persistence: StatePersistenceImpl) -> Self {
        Self {
            persistence,
            current_state: None,
        }
    }

    /// Initialize state for this device
    ///
    /// # Errors
    ///
    /// Returns an error if loading existing state from persistence fails.
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
    ///
    /// # Errors
    ///
    /// Returns an error if saving to persistence fails.
    pub fn update(&mut self, state: DeviceState) -> Result<()> {
        self.persistence.save(&state)?;
        self.current_state = Some(Arc::new(state));
        Ok(())
    }

    /// Set UI state value
    ///
    /// # Errors
    ///
    /// Returns an error if there is current state and saving to persistence fails.
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
