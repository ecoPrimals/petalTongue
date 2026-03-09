// SPDX-License-Identifier: AGPL-3.0-only
//! Niche Designer - Types and validation
//!
//! Core types for the visual niche editor and deployment UI.

/// Validation result for niche design
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// All requirements met
    Valid,
    /// Missing required primals
    MissingRequirements(Vec<String>),
    /// Resource constraints not met
    InsufficientResources(String),
    /// Configuration conflicts
    Conflicts(Vec<String>),
}
