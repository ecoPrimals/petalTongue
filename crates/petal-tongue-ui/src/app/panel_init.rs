// SPDX-License-Identifier: AGPL-3.0-or-later
//! Panel registry setup and custom panel instantiation from scenario config.

use std::sync::Arc;

use crate::panel_registry::{PanelInstance, PanelRegistry};
#[cfg(feature = "doom")]
use crate::panels::create_doom_factory;

/// Build the panel registry and any custom panels requested by the scenario.
pub(super) fn create_panel_registry_and_panels(
    scenario: Option<&crate::scenario::Scenario>,
) -> (PanelRegistry, Vec<Box<dyn PanelInstance>>) {
    let mut panel_registry = PanelRegistry::new();
    #[cfg(feature = "doom")]
    panel_registry.register(create_doom_factory());
    panel_registry.register(Arc::new(crate::panels::MetricsPanelFactory::new()));
    panel_registry.register(Arc::new(crate::panels::ProprioceptionPanelFactory::new()));

    tracing::info!("✅ Panel registry initialized");
    tracing::info!(
        "   Available panel types: {:?}",
        panel_registry.available_types()
    );

    let mut custom_panels: Vec<Box<dyn PanelInstance>> = Vec::new();
    if let Some(s) = scenario {
        for panel_config in &s.ui_config.custom_panels {
            match panel_registry.create(panel_config) {
                Ok(panel) => {
                    tracing::info!(
                        "✅ Created custom panel: {} (type: {})",
                        panel_config.title,
                        panel_config.panel_type
                    );
                    custom_panels.push(panel);
                }
                Err(e) => {
                    tracing::error!("❌ Failed to create panel '{}': {}", panel_config.title, e);
                }
            }
        }
    }
    tracing::info!(
        "✅ Custom panels initialized: {} panels",
        custom_panels.len()
    );

    (panel_registry, custom_panels)
}
