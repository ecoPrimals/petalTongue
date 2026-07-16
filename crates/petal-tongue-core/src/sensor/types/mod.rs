// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sensor types: trait, capabilities, events, input types.

mod capabilities;
mod error;
mod event;
mod gesture;
mod input;

pub use capabilities::{SensorCapabilities, SensorCapability, SensorType};
pub use error::SensorError;
pub use event::SensorEvent;
pub use gesture::{GestureDirection, GestureType};
pub use input::{Key, Modifiers, MouseButton};

use std::time::Instant;

/// Universal sensor trait - any input device implements this
pub trait Sensor: Send + Sync {
    /// Get sensor capabilities
    fn capabilities(&self) -> &SensorCapabilities;

    /// Check if sensor is currently available
    fn is_available(&self) -> bool;

    /// Poll for new events (non-blocking)
    async fn poll_events(&mut self) -> Result<Vec<SensorEvent>, SensorError>;

    /// Get last activity timestamp
    fn last_activity(&self) -> Option<Instant>;

    /// Get sensor name (for logging/debugging)
    fn name(&self) -> &str;
}
