// SPDX-License-Identifier: AGPL-3.0-only
//! Form validation logic.

use super::field::{Field, FieldType, ValidationError};

/// Validate a single field's value. Returns validation errors (may be multiple).
pub fn validate_field<T>(field: &Field<T>, value: Option<&str>) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Check required fields
    if field.required && value.unwrap_or_default().trim().is_empty() {
        errors.push(ValidationError {
            field_id: field.id.clone(),
            message: format!("{} is required", field.label),
        });
        return errors;
    }

    // Type-specific validation
    if let Some(value) = value {
        match &field.field_type {
            FieldType::Text {
                max_length,
                pattern,
                ..
            } => {
                if let Some(pattern_str) = pattern {
                    if !value.is_empty() {
                        let is_valid = validate_pattern(value, pattern_str);
                        if !is_valid {
                            errors.push(ValidationError {
                                field_id: field.id.clone(),
                                message: format!("{} format is invalid", field.label),
                            });
                        }
                    }
                }
                if let Some(max) = max_length {
                    if value.len() > *max {
                        errors.push(ValidationError {
                            field_id: field.id.clone(),
                            message: format!("{} must be at most {} characters", field.label, max),
                        });
                    }
                }
            }
            FieldType::TextArea { max_length, .. } => {
                if let Some(max) = max_length {
                    if value.len() > *max {
                        errors.push(ValidationError {
                            field_id: field.id.clone(),
                            message: format!("{} must be at most {} characters", field.label, max),
                        });
                    }
                }
            }
            FieldType::Number { min, max, .. } => {
                if let Ok(num) = value.parse::<f64>() {
                    if let Some(min_val) = min {
                        if num < *min_val {
                            errors.push(ValidationError {
                                field_id: field.id.clone(),
                                message: format!("{} must be at least {}", field.label, min_val),
                            });
                        }
                    }
                    if let Some(max_val) = max {
                        if num > *max_val {
                            errors.push(ValidationError {
                                field_id: field.id.clone(),
                                message: format!("{} must be at most {}", field.label, max_val),
                            });
                        }
                    }
                } else if !value.is_empty() {
                    errors.push(ValidationError {
                        field_id: field.id.clone(),
                        message: format!("{} must be a valid number", field.label),
                    });
                }
            }
            FieldType::Integer { min, max, .. } => {
                if let Ok(num) = value.parse::<i64>() {
                    if let Some(min_val) = min {
                        if num < *min_val {
                            errors.push(ValidationError {
                                field_id: field.id.clone(),
                                message: format!("{} must be at least {}", field.label, min_val),
                            });
                        }
                    }
                    if let Some(max_val) = max {
                        if num > *max_val {
                            errors.push(ValidationError {
                                field_id: field.id.clone(),
                                message: format!("{} must be at most {}", field.label, max_val),
                            });
                        }
                    }
                } else if !value.is_empty() {
                    errors.push(ValidationError {
                        field_id: field.id.clone(),
                        message: format!("{} must be a valid integer", field.label),
                    });
                }
            }
            FieldType::Select { options, .. } => {
                if !value.is_empty() && !options.iter().any(|o| o.as_str() == value) {
                    errors.push(ValidationError {
                        field_id: field.id.clone(),
                        message: format!("{} must be a valid option", field.label),
                    });
                }
            }
            FieldType::Radio { options, .. } => {
                if !value.is_empty() && !options.iter().any(|o| o.as_str() == value) {
                    errors.push(ValidationError {
                        field_id: field.id.clone(),
                        message: format!("{} must be a valid option", field.label),
                    });
                }
            }
            FieldType::Slider { min, max, .. } => {
                if let Ok(num) = value.parse::<f64>() {
                    if num < *min || num > *max {
                        errors.push(ValidationError {
                            field_id: field.id.clone(),
                            message: format!("{} must be between {} and {}", field.label, min, max),
                        });
                    }
                } else if !value.is_empty() {
                    errors.push(ValidationError {
                        field_id: field.id.clone(),
                        message: format!("{} must be a valid number", field.label),
                    });
                }
            }
            FieldType::Color { .. } => {
                // Color fields always have valid values (picker ensures this)
            }
            _ => {}
        }
    }

    errors
}

/// Simple pattern validation (without regex dependency)
/// TODO: Replace with regex crate for full regex support
fn validate_pattern(value: &str, _pattern: &str) -> bool {
    // For now, accept all non-empty values as valid
    !value.is_empty()
}

/// Get default value string for a field type
pub fn default_value_for_field_type(field_type: &FieldType) -> String {
    match field_type {
        FieldType::Text { default, .. } => default.clone(),
        FieldType::TextArea { default, .. } => default.clone(),
        FieldType::Number { default, .. } => default.to_string(),
        FieldType::Integer { default, .. } => default.to_string(),
        FieldType::Checkbox { default } => default.to_string(),
        FieldType::Select {
            default_index,
            options,
            ..
        } => default_index
            .and_then(|i| options.get(i).cloned())
            .unwrap_or_default(),
        FieldType::Radio {
            default_index,
            options,
        } => default_index
            .and_then(|i| options.get(i).cloned())
            .unwrap_or_default(),
        FieldType::Slider { default, .. } => default.to_string(),
        FieldType::Color { default } => format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            default.r, default.g, default.b, default.a,
        ),
        FieldType::MultiSelect { .. } => String::new(),
    }
}
