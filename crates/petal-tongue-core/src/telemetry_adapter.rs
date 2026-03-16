// SPDX-License-Identifier: AGPL-3.0-only
//! JSONL telemetry adapter for hotSpring validation data.
//!
//! Parses hotSpring's `TelemetryWriter` format (one JSON object per line) and
//! converts sections/observables into `DataBinding::TimeSeries` for visualization.
//!
//! Format: `{"t":1.234, "section":"p43_prod", "obs":"plaquette", "val":0.593}`
//! or map form: `{"t":2.0, "section":"bgk", "mass_err":1.0e-4, "energy_err":0.05}`

use crate::DataBinding;
use std::collections::HashMap;
use std::io::BufRead;

/// A single telemetry event parsed from JSONL.
#[derive(Debug, Clone)]
pub struct TelemetryEvent {
    /// Elapsed time since run start (seconds).
    pub t: f64,
    /// Physics section identifier.
    pub section: String,
    /// Observable name (from "obs" field, or key name in map form).
    pub observable: String,
    /// Measured value.
    pub value: f64,
}

/// Parsed telemetry from a JSONL source, indexed by section and observable.
#[derive(Debug, Clone, Default)]
pub struct TelemetryAdapter {
    events: Vec<TelemetryEvent>,
}

impl TelemetryAdapter {
    /// Parse JSONL content from a string.
    #[must_use]
    pub fn parse(content: &str) -> Self {
        let events = parse_jsonl(content.as_bytes());
        Self { events }
    }

    /// Parse JSONL from a reader (streaming, line-by-line).
    pub fn from_reader<R: BufRead>(reader: R) -> std::io::Result<Self> {
        let events = parse_jsonl(reader);
        Ok(Self { events })
    }

    /// Parse JSONL from a file path.
    pub fn from_file(path: &std::path::Path) -> std::io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        Self::from_reader(reader)
    }

    /// Total parsed events.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether no events were parsed.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Unique section names.
    #[must_use]
    pub fn sections(&self) -> Vec<String> {
        let mut seen = Vec::new();
        for e in &self.events {
            if !seen.contains(&e.section) {
                seen.push(e.section.clone());
            }
        }
        seen
    }

    /// Get time series for a specific section and observable.
    #[must_use]
    pub fn time_series(&self, section: &str, observable: &str) -> (Vec<f64>, Vec<f64>) {
        let mut x = Vec::new();
        let mut y = Vec::new();
        for e in &self.events {
            if e.section == section && e.observable == observable {
                x.push(e.t);
                y.push(e.value);
            }
        }
        (x, y)
    }

    /// Unique observables within a section.
    #[must_use]
    pub fn observables(&self, section: &str) -> Vec<String> {
        let mut seen = Vec::new();
        for e in &self.events {
            if e.section == section && !seen.contains(&e.observable) {
                seen.push(e.observable.clone());
            }
        }
        seen
    }

    /// Convert all telemetry into `DataBinding::TimeSeries` grouped by section.observable.
    #[must_use]
    pub fn to_data_bindings(&self) -> Vec<DataBinding> {
        let mut groups: HashMap<(String, String), (Vec<f64>, Vec<f64>)> = HashMap::new();
        for e in &self.events {
            let key = (e.section.clone(), e.observable.clone());
            let entry = groups.entry(key).or_default();
            entry.0.push(e.t);
            entry.1.push(e.value);
        }
        let mut bindings: Vec<DataBinding> = groups
            .into_iter()
            .map(
                |((section, obs), (x_values, y_values))| DataBinding::TimeSeries {
                    id: format!("{section}.{obs}"),
                    label: format!("{section} / {obs}"),
                    x_label: "Time (s)".to_string(),
                    y_label: obs,
                    unit: String::new(),
                    x_values,
                    y_values,
                },
            )
            .collect();
        bindings.sort_by(|a, b| binding_id(a).cmp(binding_id(b)));
        bindings
    }

    /// Convert a specific section into `DataBinding::TimeSeries` (one per observable).
    #[must_use]
    pub fn section_to_bindings(&self, section: &str) -> Vec<DataBinding> {
        let observables = self.observables(section);
        observables
            .into_iter()
            .map(|obs| {
                let (x_values, y_values) = self.time_series(section, &obs);
                DataBinding::TimeSeries {
                    id: format!("{section}.{obs}"),
                    label: format!("{section} / {obs}"),
                    x_label: "Time (s)".to_string(),
                    y_label: obs,
                    unit: String::new(),
                    x_values,
                    y_values,
                }
            })
            .collect()
    }
}

fn binding_id(b: &DataBinding) -> &str {
    match b {
        DataBinding::TimeSeries { id, .. }
        | DataBinding::Distribution { id, .. }
        | DataBinding::Bar { id, .. }
        | DataBinding::Gauge { id, .. }
        | DataBinding::Heatmap { id, .. }
        | DataBinding::Scatter3D { id, .. }
        | DataBinding::Scatter { id, .. }
        | DataBinding::FieldMap { id, .. }
        | DataBinding::Spectrum { id, .. }
        | DataBinding::GameScene { id, .. }
        | DataBinding::Soundscape { id, .. } => id,
    }
}

fn parse_jsonl<R: BufRead>(reader: R) -> Vec<TelemetryEvent> {
    let mut events = Vec::new();
    for line in reader.lines() {
        let Ok(line) = line else { continue };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(obj) = serde_json::from_str::<serde_json::Value>(trimmed) else {
            continue;
        };
        let serde_json::Value::Object(map) = &obj else {
            continue;
        };
        let Some(t) = map.get("t").and_then(serde_json::Value::as_f64) else {
            continue;
        };
        let section = map
            .get("section")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("default")
            .to_string();

        // "obs"+"val" form
        if let (Some(obs), Some(val)) = (
            map.get("obs").and_then(serde_json::Value::as_str),
            map.get("val").and_then(serde_json::Value::as_f64),
        ) {
            events.push(TelemetryEvent {
                t,
                section,
                observable: obs.to_string(),
                value: val,
            });
        } else {
            // Map form: every numeric key that isn't "t" is an observable
            for (key, value) in map {
                if key == "t" || key == "section" {
                    continue;
                }
                if let Some(v) = value.as_f64() {
                    events.push(TelemetryEvent {
                        t,
                        section: section.clone(),
                        observable: key.clone(),
                        value: v,
                    });
                }
            }
        }
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_obs_val_format() {
        let jsonl = r#"{"t":1.0,"section":"flow","obs":"plaquette","val":0.593}
{"t":2.0,"section":"flow","obs":"plaquette","val":0.601}"#;
        let adapter = TelemetryAdapter::parse(jsonl);
        assert_eq!(adapter.len(), 2);
        assert_eq!(adapter.sections(), vec!["flow"]);
        let (x, y) = adapter.time_series("flow", "plaquette");
        assert_eq!(x.len(), 2);
        assert!((y[0] - 0.593).abs() < 1e-9);
    }

    #[test]
    fn parse_map_format() {
        let jsonl = r#"{"t":1.0,"section":"bgk","mass_err":1.0e-4,"energy_err":0.05}"#;
        let adapter = TelemetryAdapter::parse(jsonl);
        assert_eq!(adapter.len(), 2);
        let obs = adapter.observables("bgk");
        assert!(obs.contains(&"mass_err".to_string()));
        assert!(obs.contains(&"energy_err".to_string()));
    }

    #[test]
    fn to_data_bindings_produces_timeseries() {
        let jsonl = r#"{"t":1.0,"section":"a","obs":"x","val":10.0}
{"t":2.0,"section":"a","obs":"x","val":20.0}
{"t":1.0,"section":"b","obs":"y","val":5.0}"#;
        let adapter = TelemetryAdapter::parse(jsonl);
        let bindings = adapter.to_data_bindings();
        assert_eq!(bindings.len(), 2);
        for b in &bindings {
            assert!(matches!(b, DataBinding::TimeSeries { .. }));
        }
    }

    #[test]
    fn section_to_bindings() {
        let jsonl = r#"{"t":1.0,"section":"flow","obs":"E","val":0.1}
{"t":2.0,"section":"flow","obs":"E","val":0.2}
{"t":1.0,"section":"flow","obs":"plaq","val":0.5}"#;
        let adapter = TelemetryAdapter::parse(jsonl);
        let bindings = adapter.section_to_bindings("flow");
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn empty_input() {
        let adapter = TelemetryAdapter::parse("");
        assert!(adapter.is_empty());
        assert!(adapter.sections().is_empty());
        assert!(adapter.to_data_bindings().is_empty());
    }

    #[test]
    fn malformed_lines_are_skipped() {
        let jsonl = "not json\n{\"t\":1.0,\"section\":\"a\",\"obs\":\"x\",\"val\":1.0}\n{bad";
        let adapter = TelemetryAdapter::parse(jsonl);
        assert_eq!(adapter.len(), 1);
    }

    #[test]
    fn missing_section_defaults_to_default() {
        let jsonl = r#"{"t":1.0,"obs":"x","val":1.0}"#;
        let adapter = TelemetryAdapter::parse(jsonl);
        assert_eq!(adapter.sections(), vec!["default"]);
    }
}
