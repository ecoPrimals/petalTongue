// SPDX-License-Identifier: AGPL-3.0-only
//! Rich error types for scenario loading and validation
//!
//! This module provides context-rich error types that help users understand
//! and fix issues with their scenario files.

use std::fmt::Write;
use thiserror::Error;

/// Scenario-specific errors with rich context
#[derive(Debug, Error)]
pub enum ScenarioError {
    /// A required field is missing from the scenario JSON
    #[error("Missing required field '{field}' in scenario")]
    MissingField {
        /// Name of the missing field
        field: String,
        /// Suggestion for how to fix the error
        suggestion: Option<String>,
    },

    /// A field has an invalid or unexpected value
    #[error("Invalid field value: {field} = '{value}'")]
    InvalidValue {
        /// Name of the field with invalid value
        field: String,
        /// The invalid value that was provided
        value: String,
        /// Description of what was expected
        expected: String,
    },

    /// Panel configuration is invalid or incomplete
    #[error("Panel configuration error: {message}")]
    PanelConfig {
        /// Description of the configuration error
        message: String,
        /// Index of the panel with the error (if applicable)
        panel_index: Option<usize>,
        /// Type of the panel with the error (if applicable)
        panel_type: Option<String>,
    },

    /// Capability validation failed (input/output modality)
    #[error("Capability validation error: {message}")]
    CapabilityError {
        /// Description of the capability error
        message: String,
        /// Type of capability ("output" or "input")
        capability_type: String,
        /// The invalid capability value that was provided
        invalid_value: String,
        /// List of valid capability options
        valid_options: Vec<String>,
    },

    /// Sensory system configuration is invalid
    #[error("Sensory configuration error: {message}")]
    SensoryConfigError {
        /// Description of the sensory config error
        message: String,
        /// Name of the misconfigured field
        field: String,
        /// Suggestion for fixing the error
        suggestion: String,
    },

    /// The specified panel type is not registered
    #[error("Unknown panel type '{panel_type}'")]
    UnknownPanelType {
        /// The unrecognized panel type name
        panel_type: String,
        /// List of available/registered panel types
        available_types: Vec<String>,
    },

    /// File system I/O error during scenario loading
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON parsing error during scenario deserialization
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Generic error for unclassified issues
    #[error("{0}")]
    Generic(String),
}

impl ScenarioError {
    /// Create a missing field error with suggestion
    pub fn missing_field(field: impl Into<String>, suggestion: Option<impl Into<String>>) -> Self {
        Self::MissingField {
            field: field.into(),
            suggestion: suggestion.map(std::convert::Into::into),
        }
    }

    /// Create an invalid value error
    pub fn invalid_value(
        field: impl Into<String>,
        value: impl Into<String>,
        expected: impl Into<String>,
    ) -> Self {
        Self::InvalidValue {
            field: field.into(),
            value: value.into(),
            expected: expected.into(),
        }
    }

    /// Create a panel config error
    pub fn panel_config(
        message: impl Into<String>,
        panel_index: Option<usize>,
        panel_type: Option<impl Into<String>>,
    ) -> Self {
        Self::PanelConfig {
            message: message.into(),
            panel_index,
            panel_type: panel_type.map(std::convert::Into::into),
        }
    }

    /// Create a capability error
    pub fn capability(
        message: impl Into<String>,
        capability_type: impl Into<String>,
        invalid_value: impl Into<String>,
        valid_options: Vec<impl Into<String>>,
    ) -> Self {
        Self::CapabilityError {
            message: message.into(),
            capability_type: capability_type.into(),
            invalid_value: invalid_value.into(),
            valid_options: valid_options
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
        }
    }

    /// Get user-friendly help text
    #[must_use]
    pub fn help_text(&self) -> Option<String> {
        match self {
            Self::MissingField { field, suggestion } => {
                let mut help = format!("Add the '{field}' field to your scenario JSON.");
                if let Some(sug) = suggestion {
                    let _ = write!(help, "\n\nExample:\n{sug}");
                }
                Some(help)
            }
            Self::InvalidValue {
                field, expected, ..
            } => Some(format!("The '{field}' field should be: {expected}")),
            Self::UnknownPanelType {
                panel_type,
                available_types,
            } => Some(format!(
                "Panel type '{}' is not registered.\n\nAvailable panel types:\n  {}",
                panel_type,
                available_types.join("\n  ")
            )),
            Self::CapabilityError { valid_options, .. } => {
                Some(format!("Valid options:\n  {}", valid_options.join("\n  ")))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_field_error() {
        let err = ScenarioError::missing_field("mode", Some(r#"  "mode": "doom-showcase""#));

        assert!(err.to_string().contains("Missing required field 'mode'"));
        let help = err.help_text().unwrap();
        assert!(help.contains("Add the 'mode' field"));
        assert!(help.contains("doom-showcase"));
    }

    #[test]
    fn test_invalid_value_error() {
        let err = ScenarioError::invalid_value(
            "complexity_hint",
            "invalid",
            "one of: auto, minimal, simple, standard, rich, immersive",
        );

        assert!(err.to_string().contains("Invalid field value"));
        let help = err.help_text().unwrap();
        assert!(help.contains("should be"));
    }

    #[test]
    fn test_unknown_panel_type() {
        let err = ScenarioError::UnknownPanelType {
            panel_type: "unknown_panel".to_string(),
            available_types: vec!["doom_game".to_string(), "web_view".to_string()],
        };

        assert!(
            err.to_string()
                .contains("Unknown panel type 'unknown_panel'")
        );
        let help = err.help_text().unwrap();
        assert!(help.contains("doom_game"));
        assert!(help.contains("web_view"));
    }
}
