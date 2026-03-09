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
                validate_text(value, field, max_length.as_ref(), pattern.as_deref(), &mut errors);
            }
            FieldType::TextArea { max_length, .. } => {
                validate_text_area(value, field, max_length.as_ref(), &mut errors);
            }
            FieldType::Number { min, max, .. } => {
                validate_number(value, field, min.as_ref(), max.as_ref(), &mut errors);
            }
            FieldType::Integer { min, max, .. } => {
                validate_integer(value, field, min.as_ref(), max.as_ref(), &mut errors);
            }
            FieldType::Select { options, .. } | FieldType::Radio { options, .. } => {
                validate_select_or_radio(value, field, options, &mut errors);
            }
            FieldType::Slider { min, max, .. } => {
                validate_slider(value, field, *min, *max, &mut errors);
            }
            _ => {
                // Color, MultiSelect, etc. - no validation or no single default
            }
        }
    }

    errors
}

fn validate_text<T>(
    value: &str,
    field: &Field<T>,
    max_length: Option<&usize>,
    pattern: Option<&str>,
    errors: &mut Vec<ValidationError>,
) {
    if let Some(pattern_str) = pattern {
        if !value.is_empty() && !validate_pattern(value, pattern_str) {
            errors.push(ValidationError {
                field_id: field.id.clone(),
                message: format!("{} format is invalid", field.label),
            });
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

fn validate_text_area<T>(
    value: &str,
    field: &Field<T>,
    max_length: Option<&usize>,
    errors: &mut Vec<ValidationError>,
) {
    if let Some(max) = max_length {
        if value.len() > *max {
            errors.push(ValidationError {
                field_id: field.id.clone(),
                message: format!("{} must be at most {} characters", field.label, max),
            });
        }
    }
}

fn validate_number<T>(
    value: &str,
    field: &Field<T>,
    min: Option<&f64>,
    max: Option<&f64>,
    errors: &mut Vec<ValidationError>,
) {
    match value.parse::<f64>() {
        Ok(num) => {
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
        }
        Err(_) if !value.is_empty() => {
            errors.push(ValidationError {
                field_id: field.id.clone(),
                message: format!("{} must be a valid number", field.label),
            });
        }
        Err(_) => {}
    }
}

fn validate_integer<T>(
    value: &str,
    field: &Field<T>,
    min: Option<&i64>,
    max: Option<&i64>,
    errors: &mut Vec<ValidationError>,
) {
    match value.parse::<i64>() {
        Ok(num) => {
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
        }
        Err(_) if !value.is_empty() => {
            errors.push(ValidationError {
                field_id: field.id.clone(),
                message: format!("{} must be a valid integer", field.label),
            });
        }
        Err(_) => {}
    }
}

fn validate_select_or_radio<T>(
    value: &str,
    field: &Field<T>,
    options: &[String],
    errors: &mut Vec<ValidationError>,
) {
    if !value.is_empty() && !options.iter().any(|o| o.as_str() == value) {
        errors.push(ValidationError {
            field_id: field.id.clone(),
            message: format!("{} must be a valid option", field.label),
        });
    }
}

fn validate_slider<T>(
    value: &str,
    field: &Field<T>,
    min: f64,
    max: f64,
    errors: &mut Vec<ValidationError>,
) {
    match value.parse::<f64>() {
        Ok(num) => {
            if num < min || num > max {
                errors.push(ValidationError {
                    field_id: field.id.clone(),
                    message: format!("{} must be between {} and {}", field.label, min, max),
                });
            }
        }
        Err(_) if !value.is_empty() => {
            errors.push(ValidationError {
                field_id: field.id.clone(),
                message: format!("{} must be a valid number", field.label),
            });
        }
        Err(_) => {}
    }
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
        FieldType::Text { default, .. } | FieldType::TextArea { default, .. } => default.clone(),
        FieldType::Number { default, .. } | FieldType::Slider { default, .. } => default.to_string(),
        FieldType::Integer { default, .. } => default.to_string(),
        FieldType::Checkbox { default } => default.to_string(),
        FieldType::Select {
            default_index,
            options,
            ..
        }
        | FieldType::Radio {
            default_index,
            options,
        } => default_index
            .and_then(|i| options.get(i).cloned())
            .unwrap_or_default(),
        FieldType::Color { default } => format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            default.r, default.g, default.b, default.a,
        ),
        FieldType::MultiSelect { .. } => String::new(),
    }
}
