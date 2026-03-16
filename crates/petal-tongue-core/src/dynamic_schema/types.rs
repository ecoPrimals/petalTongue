// SPDX-License-Identifier: AGPL-3.0-only

use serde::{Deserialize, Deserializer, Serialize};

use crate::error::{PetalTongueError, Result};
use std::collections::HashMap;
use std::fmt;

/// Deserialize `SchemaVersion` from either a string ("2.0.0") or struct { major, minor, patch }
pub fn deserialize_version<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<SchemaVersion>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum VersionInput {
        String(String),
        Struct { major: u32, minor: u32, patch: u32 },
    }

    let input: Option<VersionInput> = Option::deserialize(deserializer)?;
    match input {
        None => Ok(None),
        Some(VersionInput::String(s)) => SchemaVersion::parse(&s)
            .map(Some)
            .map_err(|e| serde::de::Error::custom(e.to_string())),
        Some(VersionInput::Struct {
            major,
            minor,
            patch,
        }) => Ok(Some(SchemaVersion {
            major,
            minor,
            patch,
        })),
    }
}

/// Schema version for backward/forward compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SchemaVersion {
    /// Major version (breaking changes)
    pub major: u32,
    /// Minor version (backward-compatible additions)
    pub minor: u32,
    /// Patch version (bug fixes)
    pub patch: u32,
}

impl SchemaVersion {
    /// Create a new schema version
    #[must_use]
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parse version from string (e.g., "1.2.3")
    pub fn parse(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(PetalTongueError::InvalidVersionFormat(s.to_string()));
        }

        Ok(Self {
            major: parts[0].parse().map_err(|e| {
                PetalTongueError::InvalidVersionFormat(format!("Invalid major version: {e}"))
            })?,
            minor: parts[1].parse().map_err(|e| {
                PetalTongueError::InvalidVersionFormat(format!("Invalid minor version: {e}"))
            })?,
            patch: parts[2].parse().map_err(|e| {
                PetalTongueError::InvalidVersionFormat(format!("Invalid patch version: {e}"))
            })?,
        })
    }

    /// Check if this version is compatible with another
    ///
    /// Compatible means:
    /// - Same major version (no breaking changes)
    /// - This version >= other version (forward compatible)
    #[must_use]
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.major == other.major && self >= other
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::new(1, 0, 0)
    }
}

/// Dynamic value that can represent any JSON type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DynamicValue {
    /// Null value
    Null,
    /// Boolean value
    Boolean(bool),
    /// Number value (f64 covers i64, u64, f32, f64)
    Number(f64),
    /// String value
    String(String),
    /// Array of values
    Array(Vec<Self>),
    /// Object (key-value map)
    Object(HashMap<String, Self>),
}

impl DynamicValue {
    /// Get value as string, if possible
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get value as number, if possible
    #[must_use]
    pub const fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get value as boolean, if possible
    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Get value as array, if possible
    #[must_use]
    pub fn as_array(&self) -> Option<&[Self]> {
        match self {
            Self::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Get value as object, if possible
    #[must_use]
    pub const fn as_object(&self) -> Option<&HashMap<String, Self>> {
        match self {
            Self::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Check if value is null
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Convert to `serde_json::Value` for compatibility
    #[must_use]
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            Self::Null => serde_json::Value::Null,
            Self::Boolean(b) => serde_json::Value::Bool(*b),
            Self::Number(n) => serde_json::Value::Number(
                serde_json::Number::from_f64(*n).unwrap_or_else(|| serde_json::Number::from(0)),
            ),
            Self::String(s) => serde_json::Value::String(s.clone()),
            Self::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(Self::to_json_value).collect())
            }
            Self::Object(obj) => serde_json::Value::Object(
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.to_json_value()))
                    .collect(),
            ),
        }
    }

    /// Create from `serde_json::Value`
    pub fn from_json_value(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(b) => Self::Boolean(b),
            serde_json::Value::Number(n) => Self::Number(n.as_f64().unwrap_or_default()),
            serde_json::Value::String(s) => Self::String(s),
            serde_json::Value::Array(arr) => {
                Self::Array(arr.into_iter().map(Self::from_json_value).collect())
            }
            serde_json::Value::Object(obj) => Self::Object(
                obj.into_iter()
                    .map(|(k, v)| (k, Self::from_json_value(v)))
                    .collect(),
            ),
        }
    }
}

impl From<serde_json::Value> for DynamicValue {
    fn from(value: serde_json::Value) -> Self {
        Self::from_json_value(value)
    }
}

impl From<DynamicValue> for serde_json::Value {
    fn from(value: DynamicValue) -> Self {
        value.to_json_value()
    }
}

/// Dynamic data structure that captures all fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicData {
    /// Schema version (if present)
    /// Accepts both string ("2.0.0") and struct { major, minor, patch } formats
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_version"
    )]
    pub version: Option<SchemaVersion>,

    /// All fields as dynamic values
    #[serde(flatten)]
    pub fields: HashMap<String, DynamicValue>,
}

impl DynamicData {
    /// Create empty dynamic data
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: None,
            fields: HashMap::new(),
        }
    }

    /// Create with schema version
    #[must_use]
    pub fn with_version(version: SchemaVersion) -> Self {
        Self {
            version: Some(version),
            fields: HashMap::new(),
        }
    }

    /// Get a field value
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&DynamicValue> {
        self.fields.get(key)
    }

    /// Get a field value as string
    #[must_use]
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key)?.as_str()
    }

    /// Get a field value as number
    #[must_use]
    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.get(key)?.as_f64()
    }

    /// Get a field value as boolean
    #[must_use]
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key)?.as_bool()
    }

    /// Set a field value
    pub fn set(&mut self, key: String, value: DynamicValue) {
        self.fields.insert(key, value);
    }

    /// Merge fields from another schema. For conflicting keys, values from `other` overwrite.
    pub fn merge(&mut self, other: &Self) {
        for (k, v) in &other.fields {
            self.fields.insert(k.clone(), v.clone());
        }
    }

    /// Parse from JSON string
    pub fn from_json_str(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| {
            PetalTongueError::Json(format!("Failed to parse dynamic data from JSON: {e}"))
        })
    }

    /// Parse from JSON file
    pub fn from_json_file(path: &std::path::Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path).map_err(|e| {
            PetalTongueError::Json(format!("Failed to read file {}: {e}", path.display()))
        })?;
        Self::from_json_str(&contents)
    }

    /// Convert to JSON string
    pub fn to_json_string(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| PetalTongueError::Json(format!("Failed to serialize to JSON: {e}")))
    }
}

impl Default for DynamicData {
    fn default() -> Self {
        Self::new()
    }
}
