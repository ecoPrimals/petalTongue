// SPDX-License-Identifier: AGPL-3.0-or-later
//! Instance identifiers and errors.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Unique identifier for a petalTongue instance
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstanceId(Uuid);

impl InstanceId {
    /// Generate a new unique instance ID
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the UUID as a string
    #[must_use]
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }

    /// Parse an `InstanceId` from a string representation
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not a valid UUID.
    pub fn parse(s: &str) -> Result<Self, InstanceError> {
        Ok(Self(Uuid::parse_str(s).map_err(|e| {
            InstanceError::InvalidInstanceId(format!("Invalid UUID: {e}"))
        })?))
    }
}

impl Default for InstanceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
/// Errors that can occur during instance management
#[derive(Debug, Error)]
pub enum InstanceError {
    /// Invalid instance ID
    #[error("Invalid instance ID: {0}")]
    InvalidInstanceId(String),

    /// Instance not found
    #[error("Instance not found: {0}")]
    NotFound(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Serialize error
    #[error("Serialize error: {0}")]
    SerializeError(String),

    /// Directory error
    #[error("Directory error: {0}")]
    DirectoryError(String),
}
