// SPDX-License-Identifier: AGPL-3.0-or-later

use super::migrations::{MigrationRegistry, SchemaMigrationImpl, V1ToV2Migration};
use super::types::{DynamicData, DynamicValue, SchemaVersion};
use std::collections::HashMap;

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

    assert!(v1_1_0.is_compatible_with(&v1_0_0));
    assert!(!v1_0_0.is_compatible_with(&v1_1_0));
    assert!(!v2_0_0.is_compatible_with(&v1_0_0));
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

#[test]
fn test_schema_version_parse_error() {
    assert!(SchemaVersion::parse("1.2").is_err());
    assert!(SchemaVersion::parse("1.2.3.4").is_err());
    assert!(SchemaVersion::parse("a.b.c").is_err());
}

#[test]
fn test_schema_version_default() {
    let v = SchemaVersion::default();
    assert_eq!(v.major, 1);
    assert_eq!(v.minor, 0);
    assert_eq!(v.patch, 0);
}

#[test]
fn test_dynamic_value_is_null() {
    assert!(DynamicValue::Null.is_null());
    assert!(!DynamicValue::Boolean(false).is_null());
    assert!(!DynamicValue::String(String::new()).is_null());
}

#[test]
fn test_dynamic_value_json_roundtrip() {
    let val = DynamicValue::Object(
        std::iter::once(("k".to_string(), DynamicValue::Number(42.0))).collect(),
    );
    let json_val = val.to_json_value();
    let back = DynamicValue::from_json_value(json_val);
    assert_eq!(val, back);
}

#[test]
fn test_dynamic_value_array() {
    let arr = DynamicValue::Array(vec![
        DynamicValue::Number(1.0),
        DynamicValue::String("x".to_string()),
    ]);
    assert_eq!(arr.as_array().map(<[_]>::len), Some(2));
    assert_eq!(arr.as_f64(), None);
}

#[test]
fn test_dynamic_data_with_version() {
    let data = DynamicData::with_version(SchemaVersion::new(2, 1, 0));
    assert_eq!(data.version, Some(SchemaVersion::new(2, 1, 0)));
    assert!(data.fields.is_empty());
}

#[test]
fn test_migration_registry_no_migration() {
    let registry = MigrationRegistry::new();
    let mut data = DynamicData::new();
    data.set("x".to_string(), DynamicValue::String("y".to_string()));
    let from = SchemaVersion::new(1, 0, 0);
    let to = SchemaVersion::new(1, 0, 0);
    assert!(registry.migrate(&mut data, from, to).is_ok());
    assert_eq!(data.get_str("x"), Some("y"));
}

#[test]
fn test_migration_registry_no_handler() {
    let registry = MigrationRegistry::new();
    let mut data = DynamicData::new();
    let from = SchemaVersion::new(1, 0, 0);
    let to = SchemaVersion::new(2, 0, 0);
    assert!(registry.migrate(&mut data, from, to).is_err());
}

#[test]
fn test_dynamic_data_empty_json() {
    let json = "{}";
    let data = DynamicData::from_json_str(json).unwrap();
    assert!(data.fields.is_empty());
    assert!(data.version.is_none());
}

#[test]
fn test_dynamic_data_mixed_types() {
    let json = r#"{"str":"x","num":42,"bool":true,"null":null,"arr":[1,2],"obj":{"a":1}}"#;
    let data = DynamicData::from_json_str(json).unwrap();
    assert_eq!(data.get_str("str"), Some("x"));
    assert_eq!(data.get_f64("num"), Some(42.0));
    assert_eq!(data.get_bool("bool"), Some(true));
    assert!(data.get("null").unwrap().is_null());
    assert_eq!(
        data.get("arr").and_then(|v| v.as_array()).map(<[_]>::len),
        Some(2)
    );
    assert_eq!(
        data.get("obj")
            .and_then(|v| v.as_object())
            .and_then(|o| o.get("a"))
            .and_then(DynamicValue::as_f64),
        Some(1.0)
    );
}

#[test]
fn test_schema_version_compatibility_same_major() {
    let v1_2_3 = SchemaVersion::new(1, 2, 3);
    let v1_0_0 = SchemaVersion::new(1, 0, 0);
    assert!(v1_2_3.is_compatible_with(&v1_0_0));
    assert!(!v1_0_0.is_compatible_with(&v1_2_3));
}

#[test]
fn test_schema_version_compatibility_exact_match() {
    let v = SchemaVersion::new(2, 1, 0);
    assert!(v.is_compatible_with(&v));
}

#[test]
fn test_dynamic_value_object_nested() {
    let mut inner = HashMap::new();
    inner.insert("nested".to_string(), DynamicValue::Number(42.0));
    let mut outer = HashMap::new();
    outer.insert("inner".to_string(), DynamicValue::Object(inner));
    let val = DynamicValue::Object(outer);
    let json_val = val.to_json_value();
    let restored = DynamicValue::from_json_value(json_val);
    assert_eq!(
        restored
            .as_object()
            .and_then(|o| o.get("inner"))
            .and_then(DynamicValue::as_object)
            .and_then(|o| o.get("nested"))
            .and_then(DynamicValue::as_f64),
        Some(42.0)
    );
}

#[test]
fn test_dynamic_data_from_json_with_version_string() {
    let json = r#"{"version":"2.0.1","name":"test"}"#;
    let data = DynamicData::from_json_str(json).unwrap();
    assert_eq!(data.version, Some(SchemaVersion::new(2, 0, 1)));
    assert_eq!(data.get_str("name"), Some("test"));
}

#[test]
fn test_dynamic_data_set_overwrites() {
    let mut data = DynamicData::new();
    data.set("k".to_string(), DynamicValue::String("v1".to_string()));
    data.set("k".to_string(), DynamicValue::String("v2".to_string()));
    assert_eq!(data.get_str("k"), Some("v2"));
}

#[test]
fn test_dynamic_value_number_special() {
    let zero = DynamicValue::Number(0.0);
    assert_eq!(zero.as_f64(), Some(0.0));
    let neg = DynamicValue::Number(-1.5);
    assert_eq!(neg.as_f64(), Some(-1.5));
}

#[test]
fn test_migration_registry_custom_migration() {
    let mut registry = MigrationRegistry::new();
    registry.register(SchemaMigrationImpl::V1ToV2(V1ToV2Migration));
    let mut data = DynamicData::new();
    data.set("x".to_string(), DynamicValue::String("y".to_string()));
    let from = SchemaVersion::new(1, 0, 0);
    let to = SchemaVersion::new(2, 0, 0);
    registry.migrate(&mut data, from, to).unwrap();
    assert_eq!(data.get_bool("migrated"), Some(true));
    assert_eq!(data.get_str("x"), Some("y"));
}

#[test]
fn test_dynamic_data_json_parse_error() {
    let result = DynamicData::from_json_str("{invalid}");
    assert!(result.is_err());
}

#[test]
fn test_dynamic_value_as_bool_false() {
    assert_eq!(DynamicValue::Boolean(false).as_bool(), Some(false));
}

#[test]
fn test_dynamic_data_empty_schema() {
    let data = DynamicData::new();
    assert!(data.fields.is_empty());
    assert!(data.version.is_none());
    assert!(data.get("any").is_none());
}

#[test]
fn test_dynamic_data_duplicate_field_overwrites() {
    let mut data = DynamicData::new();
    data.set("dup".to_string(), DynamicValue::String("first".to_string()));
    data.set(
        "dup".to_string(),
        DynamicValue::String("second".to_string()),
    );
    assert_eq!(data.get_str("dup"), Some("second"));
}

#[test]
fn test_dynamic_data_deeply_nested_schema() {
    let json = r#"{"level1":{"level2":{"level3":{"leaf":42}}}}"#;
    let data = DynamicData::from_json_str(json).unwrap();
    let v = data
        .get("level1")
        .and_then(DynamicValue::as_object)
        .and_then(|o| o.get("level2"))
        .and_then(DynamicValue::as_object)
        .and_then(|o| o.get("level3"))
        .and_then(DynamicValue::as_object)
        .and_then(|o| o.get("leaf"))
        .and_then(DynamicValue::as_f64);
    assert_eq!(v, Some(42.0));
}

#[test]
fn test_dynamic_data_from_json_file() {
    let dir = std::env::temp_dir().join("petal-schema-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("schema.json");
    std::fs::write(&path, r#"{"version":"1.0.0","name":"test","count":100}"#).unwrap();
    let data = DynamicData::from_json_file(&path).unwrap();
    assert_eq!(data.version, Some(SchemaVersion::new(1, 0, 0)));
    assert_eq!(data.get_str("name"), Some("test"));
    assert_eq!(data.get_f64("count"), Some(100.0));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_dynamic_data_from_json_file_nonexistent() {
    let result = DynamicData::from_json_file(std::path::Path::new("/nonexistent/schema.json"));
    assert!(result.is_err());
}

#[test]
fn test_schema_version_parse_struct_format() {
    let json = r#"{"version":{"major":3,"minor":2,"patch":1},"x":1}"#;
    let data: DynamicData = serde_json::from_str(json).unwrap();
    assert_eq!(data.version, Some(SchemaVersion::new(3, 2, 1)));
    assert_eq!(data.get_f64("x"), Some(1.0));
}

#[test]
fn test_dynamic_value_from_into_json_value() {
    let v = DynamicValue::Array(vec![DynamicValue::Boolean(true), DynamicValue::Null]);
    let j: serde_json::Value = v.clone().into();
    let back = DynamicValue::from(j);
    assert_eq!(v, back);
}

#[test]
fn test_migration_registry_skips_non_matching() {
    let mut registry = MigrationRegistry::new();
    registry.register(SchemaMigrationImpl::V1ToV2(V1ToV2Migration));
    let mut data = DynamicData::new();
    data.set("x".to_string(), DynamicValue::String("y".to_string()));
    let from = SchemaVersion::new(2, 0, 0);
    let to = SchemaVersion::new(3, 0, 0);
    assert!(registry.migrate(&mut data, from, to).is_err());
    assert_eq!(data.get_str("x"), Some("y"));
}

#[test]
fn test_dynamic_data_empty_json_array() {
    let json = r#"{"arr":[]}"#;
    let data = DynamicData::from_json_str(json).unwrap();
    let arr = data.get("arr").and_then(DynamicValue::as_array);
    assert!(arr.is_some());
    assert!(arr.unwrap().is_empty());
}

#[test]
fn test_schema_version_display() {
    let v = SchemaVersion::new(2, 5, 3);
    assert_eq!(v.to_string(), "2.5.3");
}
