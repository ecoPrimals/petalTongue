// SPDX-License-Identifier: AGPL-3.0-or-later
//! Capability parsing from discovery responses.
//!
//! Handles both flat string arrays and enriched object arrays per the
//! ecosystem dual-format standard for `capability.list` responses.

/// Parse capabilities from a discovery response, handling both flat string arrays
/// and enriched object arrays (ecosystem dual-format standard).
#[must_use]
pub fn parse_capabilities(capabilities: &[serde_json::Value]) -> Vec<String> {
    capabilities
        .iter()
        .filter_map(|v| {
            v.as_str().map(String::from).or_else(|| {
                v.as_object().and_then(|obj| {
                    obj.get("name")
                        .or_else(|| obj.get("capability"))
                        .and_then(|n| n.as_str())
                        .map(String::from)
                })
            })
        })
        .collect()
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
    fn parse_mixed_format() {
        let arr = serde_json::json!([
            "visualization.render",
            {"name": "health.check", "version": "1.0"},
            {"capability": "topology.get"}
        ]);
        let caps = arr.as_array().unwrap();
        let result = parse_capabilities(caps);
        assert_eq!(
            result,
            vec!["visualization.render", "health.check", "topology.get"]
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
}
