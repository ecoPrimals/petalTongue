// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON value extraction helpers for spring adapter payloads.

pub(super) fn extract_f64_array(value: &serde_json::Value, key: &str) -> Option<Vec<f64>> {
    value.get(key)?.as_array().map(|arr| {
        arr.iter()
            .filter_map(serde_json::Value::as_f64)
            .collect::<Vec<f64>>()
    })
}

pub(super) fn extract_string_array(value: &serde_json::Value, key: &str) -> Option<Vec<String>> {
    value.get(key)?.as_array().map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect::<Vec<String>>()
    })
}

pub(super) fn extract_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value.get(key)?.as_str().map(String::from)
}
