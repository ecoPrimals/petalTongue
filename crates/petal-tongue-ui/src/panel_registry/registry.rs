// SPDX-License-Identifier: AGPL-3.0-or-later
//! [`PanelRegistry`] — register factories and instantiate panels from scenario config.

use crate::scenario::CustomPanelConfig;
use std::collections::HashMap;
use std::sync::Arc;

use super::factory::{PanelFactory, PanelFactoryImpl};
use super::types::{PanelError, Result};

/// Registry of available panel types
pub struct PanelRegistry {
    factories: HashMap<String, Arc<PanelFactoryImpl>>,
}

impl PanelRegistry {
    /// Create a new panel registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// Register a panel factory
    pub fn register(&mut self, factory: Arc<PanelFactoryImpl>) {
        let panel_type = factory.panel_type().to_string();
        tracing::info!(
            "Registering panel type: {} - {}",
            panel_type,
            factory.description()
        );
        self.factories.insert(panel_type, factory);
    }

    /// Create a panel instance from configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the panel type is unknown or the factory fails to create the panel.
    pub fn create(&self, config: &CustomPanelConfig) -> Result<super::factory::PanelInstanceImpl> {
        let factory = self
            .factories
            .get(&config.panel_type)
            .ok_or_else(|| PanelError::UnknownType(config.panel_type.clone()))?;

        tracing::info!(
            "Creating panel: {} (type: {})",
            config.title,
            config.panel_type
        );
        factory.create(config)
    }

    /// Get list of registered panel types
    #[must_use]
    pub fn available_types(&self) -> Vec<&str> {
        self.factories
            .keys()
            .map(std::string::String::as_str)
            .collect()
    }

    /// Check if a panel type is registered
    #[must_use]
    pub fn has_type(&self, panel_type: &str) -> bool {
        self.factories.contains_key(panel_type)
    }
}

impl Default for PanelRegistry {
    fn default() -> Self {
        Self::new()
    }
}
