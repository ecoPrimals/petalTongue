// SPDX-License-Identifier: AGPL-3.0-only
//! Form primitive - Generic form builder with validation.
//!
//! Design Principles:
//! - Generic over data type T (no hardcoding)
//! - Capability-based field system (extensible)
//! - Runtime validation (not compile-time constraints)
//! - Builder pattern for ergonomic API

mod field;
mod validation;

pub use field::{Field, FieldType, FormData, ValidationError};
use std::collections::HashMap;

/// Generic form structure
#[derive(Debug)]
pub struct Form<T> {
    /// Form title
    pub title: String,

    /// Form fields
    pub fields: Vec<Field<T>>,

    /// Current form data
    pub data: FormData,

    /// Validation errors
    pub errors: Vec<ValidationError>,

    /// Whether the form has been modified
    pub modified: bool,

    /// Whether the form is currently being submitted
    pub submitting: bool,
}

impl<T> Form<T> {
    /// Create a new empty form
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            fields: Vec::new(),
            data: HashMap::new(),
            errors: Vec::new(),
            modified: false,
            submitting: false,
        }
    }

    /// Add a field to the form (builder pattern)
    #[must_use]
    pub fn with_field(mut self, field: Field<T>) -> Self {
        self.fields.push(field);
        self
    }

    /// Add multiple fields (builder pattern)
    #[must_use]
    pub fn with_fields(mut self, fields: Vec<Field<T>>) -> Self {
        self.fields.extend(fields);
        self
    }

    /// Initialize form data from an instance of T
    pub fn initialize_from(&mut self, item: &T) {
        for field in &self.fields {
            if let Some(extractor) = field.extractor {
                let value = extractor(item);
                self.data.insert(field.id.clone(), value);
            }
        }
    }

    /// Get value for a field
    #[must_use]
    pub fn get_value(&self, field_id: &str) -> Option<&String> {
        self.data.get(field_id)
    }

    /// Set value for a field
    pub fn set_value(&mut self, field_id: impl Into<String>, value: impl Into<String>) {
        self.data.insert(field_id.into(), value.into());
        self.modified = true;
    }

    /// Validate the form
    pub fn validate(&mut self) -> bool {
        self.errors.clear();

        for field in &self.fields {
            let value = self.data.get(&field.id).map(String::as_str);
            self.errors.extend(validation::validate_field(field, value));
        }

        self.errors.is_empty()
    }

    /// Check if form has errors
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get errors for a specific field
    #[must_use]
    pub fn field_errors(&self, field_id: &str) -> Vec<&ValidationError> {
        self.errors
            .iter()
            .filter(|e| e.field_id == field_id)
            .collect()
    }

    /// Reset the form to initial state
    pub fn reset(&mut self) {
        self.data.clear();
        self.errors.clear();
        self.modified = false;
        self.submitting = false;

        for field in &self.fields {
            let default_value = validation::default_value_for_field_type(&field.field_type);
            self.data.insert(field.id.clone(), default_value);
        }
    }

    /// Mark form as submitting
    pub fn start_submit(&mut self) {
        self.submitting = true;
    }

    /// Mark form as no longer submitting
    pub fn finish_submit(&mut self) {
        self.submitting = false;
    }

    /// Get field count
    #[must_use]
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    /// Find field by ID
    #[must_use]
    pub fn find_field(&self, id: &str) -> Option<&Field<T>> {
        self.fields.iter().find(|f| f.id == id)
    }

    /// Get all required fields
    #[must_use]
    pub fn required_fields(&self) -> Vec<&Field<T>> {
        self.fields.iter().filter(|f| f.required).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Color;

    #[derive(Debug, Clone)]
    struct User {
        name: String,
        age: i64,
        email: String,
        active: bool,
    }

    #[test]
    fn test_form_creation() {
        let form = Form::<User>::new("User Form")
            .with_field(Field::text("name", "Name").required())
            .with_field(Field::number("age", "Age"))
            .with_field(Field::checkbox("active", "Active"));

        assert_eq!(form.title, "User Form");
        assert_eq!(form.fields.len(), 3);
        assert_eq!(form.data.len(), 0);
    }

    #[test]
    fn test_form_validation_required() {
        let mut form =
            Form::<User>::new("Test Form").with_field(Field::text("name", "Name").required());

        assert!(!form.validate());
        assert_eq!(form.errors.len(), 1);
        assert_eq!(form.errors[0].field_id, "name");

        form.set_value("name", "Alice");
        assert!(form.validate());
        assert_eq!(form.errors.len(), 0);
    }

    #[test]
    fn test_form_validation_text_length() {
        let mut form = Form::<User>::new("Test Form").with_field(Field {
            id: "name".to_string(),
            label: "Name".to_string(),
            field_type: FieldType::Text {
                default: String::new(),
                max_length: Some(10),
                pattern: None,
            },
            required: false,
            help_text: None,
            extractor: None,
            setter: None,
        });

        form.set_value("name", "Alice");
        assert!(form.validate());

        form.set_value("name", "VeryLongNameThatExceedsLimit");
        assert!(!form.validate());
        assert_eq!(form.errors.len(), 1);
    }

    #[test]
    fn test_form_validation_number_range() {
        let mut form = Form::<User>::new("Test Form").with_field(Field {
            id: "age".to_string(),
            label: "Age".to_string(),
            field_type: FieldType::Number {
                default: 0.0,
                min: Some(0.0),
                max: Some(120.0),
                step: None,
            },
            required: false,
            help_text: None,
            extractor: None,
            setter: None,
        });

        form.set_value("age", "25");
        assert!(form.validate());

        form.set_value("age", "-5");
        assert!(!form.validate());

        form.set_value("age", "150");
        assert!(!form.validate());

        form.set_value("age", "abc");
        assert!(!form.validate());
    }

    #[test]
    fn test_form_reset() {
        let mut form = Form::<User>::new("Test Form").with_field(Field::text("name", "Name"));

        form.set_value("name", "Alice");
        assert!(form.modified);

        form.reset();
        assert!(!form.modified);
        assert_eq!(form.data.get("name"), Some(&String::new()));
    }

    #[test]
    fn test_form_field_errors() {
        let mut form = Form::<User>::new("Test Form")
            .with_field(Field::text("name", "Name").required())
            .with_field(Field::text("email", "Email").required());

        form.validate();

        let name_errors = form.field_errors("name");
        assert_eq!(name_errors.len(), 1);

        let email_errors = form.field_errors("email");
        assert_eq!(email_errors.len(), 1);
    }

    #[test]
    fn test_form_submit_state() {
        let mut form = Form::<User>::new("Test Form");

        assert!(!form.submitting);

        form.start_submit();
        assert!(form.submitting);

        form.finish_submit();
        assert!(!form.submitting);
    }

    #[test]
    fn test_validation_integer_range() {
        let mut form = Form::<User>::new("Test Form").with_field(Field {
            id: "age".to_string(),
            label: "Age".to_string(),
            field_type: FieldType::Integer {
                default: 0,
                min: Some(0),
                max: Some(120),
                step: Some(1),
            },
            required: false,
            help_text: None,
            extractor: None,
            setter: None,
        });

        form.set_value("age", "25");
        assert!(form.validate());
        assert!(form.errors.is_empty());

        form.set_value("age", "-1");
        assert!(!form.validate());
        assert!(!form.errors.is_empty());

        form.set_value("age", "200");
        assert!(!form.validate());

        form.set_value("age", "twenty");
        assert!(!form.validate());
    }

    #[test]
    #[ignore = "Requires regex crate - documented for future implementation"]
    fn test_validation_pattern_matching() {
        let mut form = Form::<User>::new("Test Form").with_field(Field {
            id: "code".to_string(),
            label: "Code".to_string(),
            field_type: FieldType::Text {
                default: String::new(),
                max_length: None,
                pattern: Some(r"^[A-Z]{3}-\d{4}$".to_string()),
            },
            required: false,
            help_text: None,
            extractor: None,
            setter: None,
        });

        form.set_value("code", "ABC-1234");
        assert!(form.validate());

        form.set_value("code", "abc-1234");
        assert!(!form.validate());

        form.set_value("code", "ABCD-123");
        assert!(!form.validate());
    }

    #[test]
    #[ignore = "Requires regex crate - documented for future implementation"]
    fn test_validation_email_pattern() {
        let mut form = Form::<User>::new("Test Form").with_field(Field {
            id: "email".to_string(),
            label: "Email".to_string(),
            field_type: FieldType::Text {
                default: String::new(),
                max_length: None,
                pattern: Some(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$".to_string()),
            },
            required: true,
            help_text: None,
            extractor: None,
            setter: None,
        });

        form.set_value("email", "user@example.com");
        assert!(form.validate());

        form.set_value("email", "userexample.com");
        assert!(!form.validate());

        form.set_value("email", "user@");
        assert!(!form.validate());

        form.set_value("email", "");
        assert!(!form.validate());
    }

    #[test]
    #[ignore = "Requires regex crate - documented for future implementation"]
    fn test_validation_url_pattern() {
        let mut form = Form::<User>::new("Test Form").with_field(Field {
            id: "website".to_string(),
            label: "Website".to_string(),
            field_type: FieldType::Text {
                default: String::new(),
                max_length: None,
                pattern: Some(r"^https?://[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}(/.*)?$".to_string()),
            },
            required: false,
            help_text: None,
            extractor: None,
            setter: None,
        });

        form.set_value("website", "https://example.com");
        assert!(form.validate());

        form.set_value("website", "https://example.com/path");
        assert!(form.validate());

        form.set_value("website", "example.com");
        assert!(!form.validate());

        form.set_value("website", "");
        assert!(form.validate());
    }

    #[test]
    fn test_validation_multiple_errors() {
        let mut form = Form::<User>::new("Test Form")
            .with_field(Field::text("name", "Name").required())
            .with_field(Field::text("email", "Email").required())
            .with_field(Field {
                id: "age".to_string(),
                label: "Age".to_string(),
                field_type: FieldType::Integer {
                    default: 0,
                    min: Some(0),
                    max: Some(120),
                    step: None,
                },
                required: true,
                help_text: None,
                extractor: None,
                setter: None,
            });

        form.validate();
        assert_eq!(form.errors.len(), 3);

        form.set_value("name", "Alice");
        form.validate();
        assert_eq!(form.errors.len(), 2);

        form.set_value("email", "alice@example.com");
        form.validate();
        assert_eq!(form.errors.len(), 1);

        form.set_value("age", "30");
        form.validate();
        assert_eq!(form.errors.len(), 0);
    }

    #[test]
    fn test_form_dirty_tracking() {
        let mut form = Form::<User>::new("Test Form").with_field(Field::text("name", "Name"));

        assert!(!form.modified);

        form.set_value("name", "Alice");
        assert!(form.modified);

        form.reset();
        assert!(!form.modified);
    }

    #[test]
    fn test_validation_textarea_max_length() {
        let mut form = Form::<User>::new("Test Form").with_field(Field {
            id: "bio".to_string(),
            label: "Bio".to_string(),
            field_type: FieldType::TextArea {
                default: String::new(),
                rows: 4,
                max_length: Some(100),
            },
            required: false,
            help_text: None,
            extractor: None,
            setter: None,
        });

        form.set_value("bio", "Short bio");
        assert!(form.validate());

        let long_bio = "A".repeat(101);
        form.set_value("bio", &long_bio);
        assert!(!form.validate());
    }

    #[test]
    fn test_validation_select_field() {
        let mut form = Form::<User>::new("Test Form").with_field(
            Field::select(
                "country",
                "Country",
                vec!["USA".to_string(), "UK".to_string(), "Canada".to_string()],
            )
            .required(),
        );

        form.set_value("country", "USA");
        assert!(form.validate());

        form.set_value("country", "InvalidCountry");
        assert!(!form.validate());

        form.set_value("country", "");
        assert!(!form.validate());
    }

    #[test]
    fn test_validation_slider_range() {
        let mut form = Form::<User>::new("Test Form").with_field(Field {
            id: "volume".to_string(),
            label: "Volume".to_string(),
            field_type: FieldType::Slider {
                min: 0.0,
                max: 100.0,
                default: 50.0,
                step: Some(1.0),
            },
            required: false,
            help_text: None,
            extractor: None,
            setter: None,
        });

        form.set_value("volume", "75");
        assert!(form.validate());

        form.set_value("volume", "-10");
        assert!(!form.validate());

        form.set_value("volume", "150");
        assert!(!form.validate());
    }

    #[test]
    fn test_field_error_retrieval() {
        let mut form = Form::<User>::new("Test Form")
            .with_field(Field::text("name", "Name").required())
            .with_field(Field::text("email", "Email").required());

        form.validate();

        let name_errors = form.field_errors("name");
        assert!(!name_errors.is_empty());

        let email_errors = form.field_errors("email");
        assert!(!email_errors.is_empty());

        let other_errors = form.field_errors("nonexistent");
        assert!(other_errors.is_empty());

        form.set_value("name", "Alice");
        form.validate();

        let name_errors_after = form.field_errors("name");
        assert!(name_errors_after.is_empty());
    }

    #[test]
    fn test_form_builder_pattern() {
        let form = Form::<User>::new("Test Form")
            .with_field(Field::text("name", "Name"))
            .with_field(Field::number("age", "Age"))
            .with_field(Field::checkbox("active", "Active"));

        assert_eq!(form.fields.len(), 3);
        assert_eq!(form.title, "Test Form");
    }

    #[test]
    fn test_radio_button_validation() {
        let mut form = Form::<User>::new("Test Form").with_field(Field {
            id: "gender".to_string(),
            label: "Gender".to_string(),
            field_type: FieldType::Radio {
                options: vec![
                    "Male".to_string(),
                    "Female".to_string(),
                    "Other".to_string(),
                ],
                default_index: None,
            },
            required: true,
            help_text: None,
            extractor: None,
            setter: None,
        });

        assert!(!form.validate());

        form.set_value("gender", "Male");
        assert!(form.validate());

        form.set_value("gender", "Invalid");
        assert!(!form.validate());
    }

    #[test]
    fn test_color_picker_validation() {
        let mut form = Form::<User>::new("Test Form").with_field(Field {
            id: "color".to_string(),
            label: "Color".to_string(),
            field_type: FieldType::Color {
                default: Color::rgb(255, 255, 255),
            },
            required: false,
            help_text: None,
            extractor: None,
            setter: None,
        });

        assert!(form.validate());

        form.set_value("color", "#FF0000");
        assert!(form.validate());
    }
}
