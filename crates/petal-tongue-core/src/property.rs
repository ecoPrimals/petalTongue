//! Generic property system for universal node representation
//!
//! This module provides a completely generic property system that makes
//! NO assumptions about ecosystem-specific data. petalTongue core only
//! knows about generic properties - ecosystem-specific meanings are
//! composed through adapters.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Generic property value that can represent any data type
///
/// This is intentionally simple and makes no assumptions about what
/// the data means. Ecosystem-specific adapters interpret these values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    /// String value
    String(String),
    /// Numeric value (f64 to support integers and floats)
    Number(f64),
    /// Boolean value
    Boolean(bool),
    /// Nested object (recursive properties)
    Object(HashMap<String, PropertyValue>),
    /// Array of values
    Array(Vec<PropertyValue>),
    /// Null/None value
    Null,
}

impl PropertyValue {
    /// Try to get as string
    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            PropertyValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get as number
    #[must_use]
    pub fn as_number(&self) -> Option<f64> {
        match self {
            PropertyValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Try to get as u8 (for things like trust levels)
    #[must_use]
    pub fn as_u8(&self) -> Option<u8> {
        self.as_number().and_then(|n| {
            if (0.0..=255.0).contains(&n) {
                // Range is already validated, so cast is safe
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let value = n as u8;
                Some(value)
            } else {
                None
            }
        })
    }

    /// Try to get as boolean
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PropertyValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get as object
    #[must_use]
    pub fn as_object(&self) -> Option<&HashMap<String, PropertyValue>> {
        match self {
            PropertyValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Try to get as array
    #[must_use]
    pub fn as_array(&self) -> Option<&Vec<PropertyValue>> {
        match self {
            PropertyValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Check if this is null
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, PropertyValue::Null)
    }
}

/// Properties map - just a type alias for clarity
pub type Properties = HashMap<String, PropertyValue>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_value_string() {
        let val = PropertyValue::String("test".to_string());
        assert_eq!(val.as_string(), Some("test"));
        assert_eq!(val.as_number(), None);
    }

    #[test]
    fn test_property_value_number() {
        let val = PropertyValue::Number(42.5);
        assert_eq!(val.as_number(), Some(42.5));
        assert_eq!(val.as_string(), None);
    }

    #[test]
    fn test_property_value_u8() {
        let val = PropertyValue::Number(3.0);
        assert_eq!(val.as_u8(), Some(3));

        let val = PropertyValue::Number(256.0);
        assert_eq!(val.as_u8(), None);
    }

    #[test]
    fn test_property_value_boolean() {
        let val = PropertyValue::Boolean(true);
        assert_eq!(val.as_bool(), Some(true));
    }

    #[test]
    fn test_property_value_object() {
        let mut obj = HashMap::new();
        obj.insert(
            "key".to_string(),
            PropertyValue::String("value".to_string()),
        );
        let val = PropertyValue::Object(obj.clone());
        assert_eq!(val.as_object(), Some(&obj));
    }

    #[test]
    fn test_property_value_null() {
        let val = PropertyValue::Null;
        assert!(val.is_null());
    }

    #[test]
    fn test_serialization() {
        let mut props = HashMap::new();
        props.insert(
            "name".to_string(),
            PropertyValue::String("test".to_string()),
        );
        props.insert("count".to_string(), PropertyValue::Number(42.0));
        props.insert("active".to_string(), PropertyValue::Boolean(true));

        let json = serde_json::to_string(&props).unwrap();
        let deserialized: HashMap<String, PropertyValue> = serde_json::from_str(&json).unwrap();

        assert_eq!(props, deserialized);
    }
}
