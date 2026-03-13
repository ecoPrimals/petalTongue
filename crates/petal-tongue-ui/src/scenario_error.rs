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

    #[test]
    fn test_panel_config_error() {
        let err = ScenarioError::panel_config("Invalid layout", Some(2), Some("scatter_plot"));
        assert!(err.to_string().contains("Panel configuration error"));
        assert!(err.to_string().contains("Invalid layout"));
        if let ScenarioError::PanelConfig {
            panel_index,
            panel_type,
            ..
        } = &err
        {
            assert_eq!(*panel_index, Some(2));
            assert_eq!(panel_type.as_deref(), Some("scatter_plot"));
        } else {
            panic!("Expected PanelConfig");
        }
    }

    #[test]
    fn test_panel_config_no_index() {
        let err = ScenarioError::panel_config("Missing size", None::<usize>, None::<String>);
        assert!(err.to_string().contains("Missing size"));
        if let ScenarioError::PanelConfig {
            panel_index,
            panel_type,
            ..
        } = &err
        {
            assert!(panel_index.is_none());
            assert!(panel_type.is_none());
        }
    }

    #[test]
    fn test_capability_error() {
        let err = ScenarioError::capability(
            "Invalid modality",
            "output",
            "haptic",
            vec!["visual", "auditory", "textual"],
        );
        assert!(err.to_string().contains("Capability validation error"));
        let help = err.help_text().unwrap();
        assert!(help.contains("visual"));
        assert!(help.contains("auditory"));
        assert!(help.contains("textual"));
    }

    #[test]
    fn test_capability_error_fields() {
        let err =
            ScenarioError::capability("Bad input", "input", "brainwave", vec!["keyboard", "mouse"]);
        if let ScenarioError::CapabilityError {
            capability_type,
            invalid_value,
            valid_options,
            ..
        } = &err
        {
            assert_eq!(capability_type, "input");
            assert_eq!(invalid_value, "brainwave");
            assert_eq!(valid_options, &["keyboard", "mouse"]);
        } else {
            panic!("Expected CapabilityError");
        }
    }

    #[test]
    fn test_sensory_config_error() {
        let err = ScenarioError::SensoryConfigError {
            message: "Rate too high".to_string(),
            field: "poll_rate".to_string(),
            suggestion: "Use 60 or lower".to_string(),
        };
        assert!(err.to_string().contains("Sensory configuration error"));
        assert!(err.to_string().contains("Rate too high"));
    }

    #[test]
    fn test_generic_error() {
        let err = ScenarioError::Generic("Something went wrong".to_string());
        assert_eq!(err.to_string(), "Something went wrong");
        assert!(err.help_text().is_none());
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err: ScenarioError = io_err.into();
        assert!(err.to_string().contains("IO error"));
        assert!(err.help_text().is_none());
    }

    #[test]
    fn test_json_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let err: ScenarioError = json_err.into();
        assert!(err.to_string().contains("JSON parsing error"));
        assert!(err.help_text().is_none());
    }

    #[test]
    fn test_missing_field_no_suggestion() {
        let err = ScenarioError::missing_field("title", None::<String>);
        let help = err.help_text().unwrap();
        assert!(help.contains("Add the 'title' field"));
        assert!(!help.contains("Example"));
    }

    #[test]
    fn test_help_text_panel_config_none() {
        let err = ScenarioError::panel_config("test", None::<usize>, None::<String>);
        assert!(err.help_text().is_none());
    }

    #[test]
    fn test_help_text_sensory_config_none() {
        let err = ScenarioError::SensoryConfigError {
            message: "test".to_string(),
            field: "f".to_string(),
            suggestion: "s".to_string(),
        };
        assert!(err.help_text().is_none());
    }
}
