// SPDX-License-Identifier: AGPL-3.0-or-later
//! Visualization and neural API provider discovery and wiring.

use crate::tutorial_mode::TutorialMode;
use petal_tongue_discovery::{
    NeuralApiProvider, VisualizationDataProvider, discover_visualization_providers,
};
use std::sync::Arc;

/// Resolve visualization data providers from scenario, tutorial mode, or live discovery.
pub(super) fn discover_data_providers(
    scenario: Option<&crate::scenario::Scenario>,
    scenario_path_for_provider: Option<&std::path::PathBuf>,
    tutorial_mode: &TutorialMode,
    runtime: &tokio::runtime::Runtime,
) -> Vec<Box<dyn VisualizationDataProvider>> {
    if let (Some(_scenario), Some(path)) = (scenario, scenario_path_for_provider) {
        tracing::info!("📋 Scenario mode: Loading primals with dynamic schema");
        match petal_tongue_discovery::DynamicScenarioProvider::from_file(path) {
            Ok(provider) => {
                if let Some(version) = provider.version() {
                    tracing::info!("   Schema version: {}", version);
                }
                vec![Box::new(provider) as Box<dyn VisualizationDataProvider>]
            }
            Err(e) => {
                tracing::error!("Failed to create dynamic scenario provider: {}", e);
                tracing::info!("Falling back to static provider...");
                match petal_tongue_discovery::ScenarioVisualizationProvider::from_file(path) {
                    Ok(provider) => vec![Box::new(provider) as Box<dyn VisualizationDataProvider>],
                    Err(e2) => {
                        tracing::error!("Static provider also failed: {}", e2);
                        vec![]
                    }
                }
            }
        }
    } else if tutorial_mode.is_enabled() {
        tracing::info!("📚 Tutorial mode: Using demonstration data");
        #[cfg(feature = "mock")]
        {
            vec![
                Box::new(petal_tongue_discovery::DemoVisualizationProvider::new())
                    as Box<dyn VisualizationDataProvider>,
            ]
        }
        #[cfg(not(feature = "mock"))]
        {
            tracing::info!("💡 Mock feature disabled - start with --features mock for demo data");
            vec![]
        }
    } else {
        runtime.block_on(async {
            match discover_visualization_providers().await {
                Ok(providers) => {
                    if providers.is_empty() {
                        tracing::warn!("No visualization providers discovered");
                        if crate::tutorial_mode::should_fallback(0) {
                            #[cfg(feature = "mock")]
                            {
                                tracing::info!("💡 Using tutorial data as graceful fallback");
                                vec![Box::new(
                                    petal_tongue_discovery::DemoVisualizationProvider::new(),
                                )
                                    as Box<dyn VisualizationDataProvider>]
                            }
                            #[cfg(not(feature = "mock"))]
                            {
                                tracing::info!("💡 Use --features mock for demo data fallback");
                                vec![]
                            }
                        } else {
                            vec![]
                        }
                    } else {
                        tracing::info!(
                            "✅ Discovered {} visualization data provider(s)",
                            providers.len()
                        );
                        for provider in &providers {
                            let metadata = provider.get_metadata();
                            tracing::info!(
                                "  - {} at {} (protocol: {})",
                                metadata.name,
                                metadata.endpoint,
                                metadata.protocol
                            );
                        }
                        providers
                    }
                }
                Err(e) => {
                    tracing::error!("Provider discovery failed: {}", e);
                    if crate::tutorial_mode::should_fallback(0) {
                        #[cfg(feature = "mock")]
                        {
                            tracing::info!("💡 Using tutorial data as graceful fallback");
                            vec![
                                Box::new(petal_tongue_discovery::DemoVisualizationProvider::new())
                                    as Box<dyn VisualizationDataProvider>,
                            ]
                        }
                        #[cfg(not(feature = "mock"))]
                        {
                            tracing::info!("💡 Use --features mock for demo data fallback");
                            vec![]
                        }
                    } else {
                        vec![]
                    }
                }
            }
        })
    }
}

/// Discover optional Neural API provider (central coordinator).
pub(super) fn discover_neural_api(
    runtime: &tokio::runtime::Runtime,
) -> Option<Arc<NeuralApiProvider>> {
    tracing::info!("🧠 Attempting Neural API discovery (central coordinator)...");
    runtime.block_on(async {
        match NeuralApiProvider::discover(None).await {
            Ok(provider) => {
                tracing::info!("✅ Neural API connected - using as primary provider");
                Some(Arc::new(provider))
            }
            Err(e) => {
                tracing::info!("Neural API not available: {} (graceful degradation)", e);
                None
            }
        }
    })
}
