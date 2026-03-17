// SPDX-License-Identifier: AGPL-3.0-or-later
//! Capability parsing from discovery responses.
//!
//! Handles all 4 ecosystem capability formats (airSpring V087 standard):
//!
//! - **Format A**: Flat string array `["visualization.render", "health.check"]`
//! - **Format B**: Enriched objects `[{"name": "visualization.render", "version": "1.0"}]`
//! - **Format C**: Nested `{"capabilities": {"list": [...]}}` (double-nested)
//! - **Format D**: Result-wrapped `{"result": [...]}` or `{"result": {"capabilities": [...]}}`

/// Parse capabilities from a flat or enriched array of values.
///
/// Handles Format A (flat strings), Format B (objects with `name` or `capability` key),
/// and mixed arrays containing both.
#[must_use]
pub fn parse_capabilities(capabilities: &[serde_json::Value]) -> Vec<String> {
    capabilities
        .iter()
        .filter_map(|v| {
            v.as_str().map(String::from).or_else(|| {
                v.as_object().and_then(|obj| {
                    obj.get("name")
                        .or_else(|| obj.get("capability"))
                        .or_else(|| obj.get("id"))
                        .and_then(|n| n.as_str())
                        .map(String::from)
                })
            })
        })
        .collect()
}

/// Extract and parse capabilities from a full JSON-RPC response body.
///
/// Handles all 4 ecosystem formats by unwrapping nested structures:
/// - Direct array at `response["capabilities"]`
/// - Result-wrapped: `response["result"]["capabilities"]`
/// - Double-nested: `response["capabilities"]["list"]`
/// - Result + double-nested: `response["result"]["capabilities"]["list"]`
/// - Plain result array: `response["result"]` if it's an array
#[must_use]
pub fn parse_capabilities_from_response(response: &serde_json::Value) -> Vec<String> {
    let candidates = [
        response.get("capabilities"),
        response.get("result").and_then(|r| r.get("capabilities")),
    ];

    for candidate in candidates.into_iter().flatten() {
        if let Some(arr) = candidate.as_array() {
            return parse_capabilities(arr);
        }
        if let Some(nested) = candidate.get("list").and_then(|l| l.as_array()) {
            return parse_capabilities(nested);
        }
    }

    if let Some(result_arr) = response.get("result").and_then(|r| r.as_array()) {
        return parse_capabilities(result_arr);
    }

    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_flat_string_array() {
        let arr = serde_json::json!(["visualization.render", "health.check"]);
        let caps = arr.as_array().unwrap();
        let result = parse_capabilities(caps);
        assert_eq!(result, vec!["visualization.render", "health.check"]);
    }

    #[test]
    fn parse_enriched_object_array_with_name() {
        let arr = serde_json::json!([
            {"name": "visualization.render", "version": "1.0"},
            {"name": "health.check", "version": "1.0"}
        ]);
        let caps = arr.as_array().unwrap();
        let result = parse_capabilities(caps);
        assert_eq!(result, vec!["visualization.render", "health.check"]);
    }

    #[test]
    fn parse_enriched_object_array_with_capability() {
        let arr = serde_json::json!([
            {"capability": "visualization.render", "version": "1.0"},
            {"capability": "health.check"}
        ]);
        let caps = arr.as_array().unwrap();
        let result = parse_capabilities(caps);
        assert_eq!(result, vec!["visualization.render", "health.check"]);
    }

    #[test]
    fn parse_enriched_object_with_id() {
        let arr = serde_json::json!([
            {"id": "gpu.dispatch", "version": "0.1.0"}
        ]);
        let caps = arr.as_array().unwrap();
        let result = parse_capabilities(caps);
        assert_eq!(result, vec!["gpu.dispatch"]);
    }

    #[test]
    fn parse_mixed_format() {
        let arr = serde_json::json!([
            "visualization.render",
            {"name": "health.check", "version": "1.0"},
            {"capability": "topology.get"},
            {"id": "gpu.dispatch"}
        ]);
        let caps = arr.as_array().unwrap();
        let result = parse_capabilities(caps);
        assert_eq!(
            result,
            vec![
                "visualization.render",
                "health.check",
                "topology.get",
                "gpu.dispatch"
            ]
        );
    }

    #[test]
    fn parse_empty_array() {
        let arr = serde_json::json!([]);
        let caps = arr.as_array().unwrap();
        let result = parse_capabilities(caps);
        assert!(result.is_empty());
    }

    #[test]
    fn parse_filters_invalid_elements() {
        let arr = serde_json::json!(["valid", 123, null, "also-valid", {}]);
        let caps = arr.as_array().unwrap();
        let result = parse_capabilities(caps);
        assert_eq!(result, vec!["valid", "also-valid"]);
    }

    #[test]
    fn parse_prefers_name_over_capability() {
        let arr = serde_json::json!([
            {"name": "from_name", "capability": "from_capability"}
        ]);
        let caps = arr.as_array().unwrap();
        let result = parse_capabilities(caps);
        assert_eq!(result, vec!["from_name"]);
    }

    #[test]
    fn from_response_direct_capabilities() {
        let resp = serde_json::json!({"capabilities": ["a", "b"]});
        assert_eq!(parse_capabilities_from_response(&resp), vec!["a", "b"]);
    }

    #[test]
    fn from_response_result_wrapped() {
        let resp = serde_json::json!({"result": {"capabilities": ["x", "y"]}});
        assert_eq!(parse_capabilities_from_response(&resp), vec!["x", "y"]);
    }

    #[test]
    fn from_response_double_nested() {
        let resp = serde_json::json!({"capabilities": {"list": ["c", "d"]}});
        assert_eq!(parse_capabilities_from_response(&resp), vec!["c", "d"]);
    }

    #[test]
    fn from_response_result_plus_double_nested() {
        let resp = serde_json::json!({"result": {"capabilities": {"list": ["e"]}}});
        assert_eq!(parse_capabilities_from_response(&resp), vec!["e"]);
    }

    #[test]
    fn from_response_plain_result_array() {
        let resp = serde_json::json!({"result": ["f", "g"]});
        assert_eq!(parse_capabilities_from_response(&resp), vec!["f", "g"]);
    }

    #[test]
    fn from_response_enriched_objects_in_result() {
        let resp = serde_json::json!({
            "result": {
                "capabilities": [
                    {"name": "visualization.render", "version": "1.0"},
                    "health.check"
                ]
            }
        });
        let caps = parse_capabilities_from_response(&resp);
        assert_eq!(caps, vec!["visualization.render", "health.check"]);
    }

    #[test]
    fn from_response_empty() {
        assert!(parse_capabilities_from_response(&serde_json::json!({})).is_empty());
        assert!(parse_capabilities_from_response(&serde_json::json!(null)).is_empty());
    }
}
