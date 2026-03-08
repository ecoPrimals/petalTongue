// SPDX-License-Identifier: AGPL-3.0-only
//! Dynamic schema system for live-evolving data structures
//!
//! This module provides schema-agnostic data handling that enables petalTongue
//! to adapt to changing JSON schemas without recompilation.
//!
//! # Philosophy
//!
//! **Code should not know the future.** Data structures evolve over time:
//! - New fields are added
//! - Old fields are deprecated
//! - Types change (string → enum, number → object)
//!
//! Traditional approach (BRITTLE):
//! ```rust,ignore
//! #[derive(Deserialize)]
//! struct Primal {
//!     id: String,
//!     name: String,
//!     // ❌ What if a new field "tier" is added tomorrow?
//!     // ❌ Requires recompilation!
//! }
//! ```
//!
//! Dynamic approach (ADAPTIVE):
//! ```rust,ignore
//! let primal = DynamicData::from_json(json)?;
//! // ✅ Captures ALL fields (known and unknown)
//! // ✅ No recompilation needed
//! // ✅ Can migrate between versions
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Deserialize `SchemaVersion` from either a string ("2.0.0") or struct { major, minor, patch }
fn deserialize_version<'de, D>(deserializer: D) -> Result<Option<SchemaVersion>, D::Error>
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
            .map_err(serde::de::Error::custom),
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
            anyhow::bail!("Invalid version format: {s}");
        }

        Ok(Self {
            major: parts[0].parse().context("Invalid major version")?,
            minor: parts[1].parse().context("Invalid minor version")?,
            patch: parts[2].parse().context("Invalid patch version")?,
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
    Array(Vec<DynamicValue>),
    /// Object (key-value map)
    Object(HashMap<String, DynamicValue>),
}

impl DynamicValue {
    /// Get value as string, if possible
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            DynamicValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get value as number, if possible
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            DynamicValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get value as boolean, if possible
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            DynamicValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Get value as array, if possible
    #[must_use]
    pub fn as_array(&self) -> Option<&[DynamicValue]> {
        match self {
            DynamicValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Get value as object, if possible
    #[must_use]
    pub fn as_object(&self) -> Option<&HashMap<String, DynamicValue>> {
        match self {
            DynamicValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Check if value is null
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, DynamicValue::Null)
    }

    /// Convert to `serde_json::Value` for compatibility
    #[must_use]
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            DynamicValue::Null => serde_json::Value::Null,
            DynamicValue::Boolean(b) => serde_json::Value::Bool(*b),
            DynamicValue::Number(n) => serde_json::Value::Number(
                serde_json::Number::from_f64(*n).unwrap_or_else(|| serde_json::Number::from(0)),
            ),
            DynamicValue::String(s) => serde_json::Value::String(s.clone()),
            DynamicValue::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(DynamicValue::to_json_value).collect())
            }
            DynamicValue::Object(obj) => serde_json::Value::Object(
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.to_json_value()))
                    .collect(),
            ),
        }
    }

    /// Create from `serde_json::Value`
    pub fn from_json_value(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => DynamicValue::Null,
            serde_json::Value::Bool(b) => DynamicValue::Boolean(b),
            serde_json::Value::Number(n) => DynamicValue::Number(n.as_f64().unwrap_or_default()),
            serde_json::Value::String(s) => DynamicValue::String(s),
            serde_json::Value::Array(arr) => {
                DynamicValue::Array(arr.into_iter().map(Self::from_json_value).collect())
            }
            serde_json::Value::Object(obj) => DynamicValue::Object(
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

    /// Parse from JSON string
    pub fn from_json_str(json: &str) -> Result<Self> {
        serde_json::from_str(json).context("Failed to parse dynamic data from JSON")
    }

    /// Parse from JSON file
    pub fn from_json_file(path: &std::path::Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        Self::from_json_str(&contents)
    }

    /// Convert to JSON string
    pub fn to_json_string(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize to JSON")
    }
}

impl Default for DynamicData {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for schema migration
pub trait SchemaMigration {
    /// Check if this migration can handle the version upgrade
    fn can_migrate(&self, from: SchemaVersion, to: SchemaVersion) -> bool;

    /// Perform the migration
    fn migrate(&self, data: &mut DynamicData, from: SchemaVersion, to: SchemaVersion)
    -> Result<()>;
}

/// Migration registry for managing schema upgrades
#[derive(Default)]
pub struct MigrationRegistry {
    migrations: Vec<Box<dyn SchemaMigration>>,
}

impl MigrationRegistry {
    /// Create a new migration registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    /// Register a migration
    pub fn register(&mut self, migration: Box<dyn SchemaMigration>) {
        self.migrations.push(migration);
    }

    /// Migrate data from one version to another
    pub fn migrate(
        &self,
        data: &mut DynamicData,
        from: SchemaVersion,
        to: SchemaVersion,
    ) -> Result<()> {
        if from == to {
            return Ok(()); // No migration needed
        }

        // Find applicable migration
        for migration in &self.migrations {
            if migration.can_migrate(from, to) {
                return migration.migrate(data, from, to);
            }
        }

        anyhow::bail!("No migration found for {from} → {to}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version_parse() {
        let v = SchemaVersion::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn test_schema_version_compatibility() {
        let v1_0_0 = SchemaVersion::new(1, 0, 0);
        let v1_1_0 = SchemaVersion::new(1, 1, 0);
        let v2_0_0 = SchemaVersion::new(2, 0, 0);

        assert!(v1_1_0.is_compatible_with(&v1_0_0)); // Forward compatible
        assert!(!v1_0_0.is_compatible_with(&v1_1_0)); // Not backward compatible
        assert!(!v2_0_0.is_compatible_with(&v1_0_0)); // Breaking change
    }

    #[test]
    fn test_dynamic_value_conversions() {
        let str_val = DynamicValue::String("test".to_string());
        assert_eq!(str_val.as_str(), Some("test"));
        assert_eq!(str_val.as_f64(), None);

        let num_val = DynamicValue::Number(42.5);
        assert_eq!(num_val.as_f64(), Some(42.5));
        assert_eq!(num_val.as_str(), None);

        let bool_val = DynamicValue::Boolean(true);
        assert_eq!(bool_val.as_bool(), Some(true));
    }

    #[test]
    fn test_dynamic_data() {
        let mut data = DynamicData::new();
        data.set("name".to_string(), DynamicValue::String("Test".to_string()));
        data.set("count".to_string(), DynamicValue::Number(42.0));

        assert_eq!(data.get_str("name"), Some("Test"));
        assert_eq!(data.get_f64("count"), Some(42.0));
        assert_eq!(data.get_str("unknown"), None);
    }

    #[test]
    fn test_dynamic_data_json_roundtrip() {
        let json = r#"{"name": "Test", "count": 42, "active": true}"#;
        let data = DynamicData::from_json_str(json).unwrap();

        assert_eq!(data.get_str("name"), Some("Test"));
        assert_eq!(data.get_f64("count"), Some(42.0));
        assert_eq!(data.get_bool("active"), Some(true));

        let json_out = data.to_json_string().unwrap();
        let data2 = DynamicData::from_json_str(&json_out).unwrap();
        assert_eq!(data2.get_str("name"), Some("Test"));
    }
}
