// SPDX-License-Identifier: AGPL-3.0-or-later
//! Data binding and threshold range types for universal visualization.
//!
//! Canonical definitions live in `petal-tongue-types` (WASM-portable).
//! This module re-exports them for backward compatibility.

pub use petal_tongue_types::{DataBinding, ThresholdRange};

#[cfg(test)]
mod tests {
    use super::{DataBinding, ThresholdRange};

    /// Verify re-exports work and types round-trip through serde.
    #[test]
    fn reexport_round_trip() {
        let json = r#"{
            "channel_type": "timeseries",
            "id": "test",
            "label": "Test",
            "x_label": "X",
            "y_label": "Y",
            "unit": "u",
            "x_values": [1.0],
            "y_values": [2.0]
        }"#;
        let binding: DataBinding = serde_json::from_str(json).expect("deserialize");
        assert!(matches!(binding, DataBinding::TimeSeries { .. }));

        let tr = ThresholdRange {
            label: "Normal".to_string(),
            min: 0.0,
            max: 1.0,
            status: "normal".to_string(),
        };
        let serialized = serde_json::to_string(&tr).expect("serialize");
        assert!(serialized.contains("Normal"));
    }
}
