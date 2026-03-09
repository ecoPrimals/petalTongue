// SPDX-License-Identifier: AGPL-3.0-only
//! Form field types and builder API.

use crate::common::Color;
use std::collections::HashMap;

/// A generic form field definition
#[derive(Debug, Clone)]
pub struct Field<T> {
    /// Unique field identifier
    pub id: String,

    /// Human-readable label
    pub label: String,

    /// Field type and constraints
    pub field_type: FieldType,

    /// Whether this field is required
    pub required: bool,

    /// Help text for the user
    pub help_text: Option<String>,

    /// Function to extract value from T
    pub extractor: Option<fn(&T) -> String>,

    /// Function to set value in T
    pub setter: Option<fn(&mut T, String)>,
}

/// Field types with validation constraints
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    /// Single-line text input
    Text {
        /// Default text value
        default: String,
        /// Maximum length allowed
        max_length: Option<usize>,
        /// Validation pattern (regex)
        pattern: Option<String>,
    },

    /// Multi-line text input
    TextArea {
        /// Default text value
        default: String,
        /// Number of visible rows
        rows: usize,
        /// Maximum length allowed
        max_length: Option<usize>,
    },

    /// Numeric input with range
    Number {
        /// Default numeric value
        default: f64,
        /// Minimum allowed value
        min: Option<f64>,
        /// Maximum allowed value
        max: Option<f64>,
        /// Increment step size
        step: Option<f64>,
    },

    /// Integer input with range
    Integer {
        /// Default integer value
        default: i64,
        /// Minimum allowed value
        min: Option<i64>,
        /// Maximum allowed value
        max: Option<i64>,
        /// Increment step size
        step: Option<i64>,
    },

    /// Dropdown selection
    Select {
        /// Available options
        options: Vec<String>,
        /// Default selected index
        default_index: Option<usize>,
    },

    /// Multiple choice checkboxes
    MultiSelect {
        /// Available options
        options: Vec<String>,
        /// Default selected indices
        default_selected: Vec<usize>,
    },

    /// Single checkbox
    Checkbox {
        /// Default checked state
        default: bool,
    },

    /// Radio buttons (single choice from multiple)
    Radio {
        /// Available options
        options: Vec<String>,
        /// Default selected index
        default_index: Option<usize>,
    },

    /// Slider for numeric values
    Slider {
        /// Minimum slider value
        min: f64,
        /// Maximum slider value
        max: f64,
        /// Default slider value
        default: f64,
        /// Increment step size
        step: Option<f64>,
    },

    /// Color picker
    Color {
        /// Default color value
        default: Color,
    },
}

/// Validation error
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    /// ID of field that failed validation
    pub field_id: String,
    /// Human-readable error message
    pub message: String,
}

/// Form data - field ID to value mapping
pub type FormData = HashMap<String, String>;

impl<T> Field<T> {
    /// Create a new text field
    pub fn text(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            field_type: FieldType::Text {
                default: String::new(),
                max_length: None,
                pattern: None,
            },
            required: false,
            help_text: None,
            extractor: None,
            setter: None,
        }
    }

    /// Create a new number field
    pub fn number(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            field_type: FieldType::Number {
                default: 0.0,
                min: None,
                max: None,
                step: None,
            },
            required: false,
            help_text: None,
            extractor: None,
            setter: None,
        }
    }

    /// Create a new checkbox field
    pub fn checkbox(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            field_type: FieldType::Checkbox { default: false },
            required: false,
            help_text: None,
            extractor: None,
            setter: None,
        }
    }

    /// Create a new select field
    pub fn select(id: impl Into<String>, label: impl Into<String>, options: Vec<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            field_type: FieldType::Select {
                options,
                default_index: None,
            },
            required: false,
            help_text: None,
            extractor: None,
            setter: None,
        }
    }

    /// Mark field as required
    #[must_use]
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Add help text
    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help_text = Some(help.into());
        self
    }

    /// Add value extractor
    #[must_use]
    pub fn with_extractor(mut self, extractor: fn(&T) -> String) -> Self {
        self.extractor = Some(extractor);
        self
    }
}
