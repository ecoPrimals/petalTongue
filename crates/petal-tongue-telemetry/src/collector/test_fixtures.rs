// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::types::TelemetryEvent;

pub(super) fn event_primal_id(event: &TelemetryEvent) -> Option<&str> {
    match event {
        TelemetryEvent::PrimalDiscovered { primal_id, .. }
        | TelemetryEvent::PrimalDisappeared { primal_id, .. }
        | TelemetryEvent::HealthUpdate { primal_id, .. } => Some(primal_id),
        _ => None,
    }
}
