// SPDX-License-Identifier: AGPL-3.0-or-later

use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, Properties, PropertyValue};

pub(super) fn create_test_primal(id: &str, trust: Option<u8>, family: Option<&str>) -> PrimalInfo {
    let mut props = Properties::new();
    if let Some(t) = trust {
        props.insert(
            "trust_level".to_string(),
            PropertyValue::Number(f64::from(t)),
        );
    }
    if let Some(f) = family {
        props.insert(
            "family_id".to_string(),
            PropertyValue::String(f.to_string()),
        );
    }

    PrimalInfo {
        id: id.to_string().into(),
        name: format!("Test Primal {}", id),
        primal_type: "Test".to_string(),
        endpoint: "http://test".to_string(),
        capabilities: vec![],
        health: PrimalHealthStatus::Healthy,
        last_seen: 0,
        endpoints: None,
        metadata: None,
        properties: props,
    }
}
