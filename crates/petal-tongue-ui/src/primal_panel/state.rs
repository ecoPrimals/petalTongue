// SPDX-License-Identifier: AGPL-3.0-or-later
//! Filter/search logic and event application for the primal list (no egui).

use crate::biomeos_integration::{Health, Primal};
use crate::ui_events::UIEvent;
use tracing::{debug, info};

use super::filter::PrimalFilter;

/// Returns primals matching the current filter and search query.
pub fn filter_primals<'a>(
    primals: &'a [Primal],
    filter: PrimalFilter,
    search_query: &str,
) -> Vec<&'a Primal> {
    primals
        .iter()
        .filter(|primal| {
            let filter_match = match filter {
                PrimalFilter::All => true,
                PrimalFilter::Healthy => primal.health == Health::Healthy,
                PrimalFilter::Degraded => primal.health == Health::Degraded,
            };

            let search_match = if search_query.is_empty() {
                true
            } else {
                let query = search_query.to_lowercase();
                primal.name.to_lowercase().contains(&query)
                    || primal.id.to_lowercase().contains(&query)
                    || primal
                        .capabilities
                        .iter()
                        .any(|c| c.to_lowercase().contains(&query))
            };

            filter_match && search_match
        })
        .collect()
}

/// Applies primal-panel-specific events to the in-memory list and selection.
pub fn apply_primal_events(
    primals: &mut Vec<Primal>,
    selected: &mut Option<String>,
    events: Vec<UIEvent>,
) {
    for event in events {
        match event {
            UIEvent::PrimalDiscovered(primal) => {
                info!("📥 Primal discovered: {}", primal.name);
                primals.push(primal);
            }
            UIEvent::PrimalRemoved(primal_id) => {
                info!("📤 Primal removed: {}", primal_id);
                primals.retain(|p| p.id != primal_id);
                if selected.as_ref() == Some(&primal_id) {
                    *selected = None;
                }
            }
            UIEvent::PrimalHealthChanged(primal_id, new_health) => {
                if let Some(primal) = primals.iter_mut().find(|p| p.id == primal_id) {
                    debug!("🔄 Primal {} health changed to {:?}", primal_id, new_health);
                    primal.health = new_health;
                }
            }
            UIEvent::PrimalLoadChanged(primal_id, new_load) => {
                if let Some(primal) = primals.iter_mut().find(|p| p.id == primal_id) {
                    primal.load = new_load;
                }
            }
            _ => {}
        }
    }
}
