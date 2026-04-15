// SPDX-License-Identifier: AGPL-3.0-or-later
//! Handlers for health, capability, and topology JSON-RPC methods.

mod capabilities;
mod health;
mod identity_lifecycle;
mod provider;
mod topology;

pub use capabilities::{
    get_capabilities, handle_announce_capabilities, handle_capabilities_sensory,
    handle_capabilities_sensory_negotiate,
};
pub use health::{
    get_health, handle_health_check, handle_health_liveness, handle_health_readiness,
};
pub use identity_lifecycle::{handle_identity_get, handle_lifecycle_status};
pub use provider::handle_provider_register;
pub use topology::get_topology;

#[cfg(test)]
mod tests;
