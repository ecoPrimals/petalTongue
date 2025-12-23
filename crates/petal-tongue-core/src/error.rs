//! petalTongue error types.

use thiserror::Error;

/// Errors specific to petalTongue.
#[derive(Debug, Error)]
pub enum petalTongueError {
    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),
    
    // TODO: Add petalTongue-specific errors
    
    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),
}
