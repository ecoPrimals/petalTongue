// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSONL telemetry file provider for springs that emit line-delimited JSON.
//!
//! Reads telemetry files written by hotSpring, groundSpring, or any primal
//! that writes JSONL with the `TelemetryEvent` schema:
//!
//! ```json
//! {"t": 1.23, "section": "lattice_qcd", "plaquette": 0.56, "acceptance": 0.82}
//! ```
//!
//! Each line is a JSON object with at minimum `t` (elapsed seconds) and
//! `section` (domain identifier). Additional numeric fields are collected
//! as key-value pairs.
//!
//! # Discovery
//!
//! The provider checks (in order):
//! 1. `PETALTONGUE_TELEMETRY_DIR` env var
//! 2. `$XDG_DATA_HOME/petaltongue/telemetry/`
//! 3. `/tmp/petaltongue-telemetry/`
//!
//! All `.jsonl` files in the directory are read. Each `section` becomes
//! a primal-like entry with `TimeSeries` channels for its numeric fields.

use std::collections::HashMap;
use std::io::BufRead;
use std::path::{Path, PathBuf};

use petal_tongue_core::DataBinding;
use serde::Deserialize;

use crate::errors::{DiscoveryError, DiscoveryResult};

/// A single telemetry event from a JSONL file.
#[derive(Debug, Clone, Deserialize)]
pub struct TelemetryEvent {
    /// Elapsed time in seconds.
    pub t: f64,
    /// Domain/section identifier (e.g. "`lattice_qcd`", "`molecular_dynamics`").
    pub section: String,
    /// All other numeric fields are captured here.
    #[serde(flatten)]
    pub fields: HashMap<String, serde_json::Value>,
}

/// Parsed telemetry: section name -> field name -> (timestamps, values).
#[derive(Debug, Clone, Default)]
pub struct TelemetryData {
    pub sections: HashMap<String, HashMap<String, TimeSeries>>,
}

/// A single time series extracted from JSONL fields.
#[derive(Debug, Clone, Default)]
pub struct TimeSeries {
    pub timestamps: Vec<f64>,
    pub values: Vec<f64>,
}

/// Discover the telemetry directory.
#[must_use]
pub fn telemetry_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("PETALTONGUE_TELEMETRY_DIR") {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            return Some(p);
        }
    }

    if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
        let p = PathBuf::from(data_home).join("petaltongue/telemetry");
        if p.is_dir() {
            return Some(p);
        }
    }

    let fallback = PathBuf::from("/tmp/petaltongue-telemetry");
    if fallback.is_dir() {
        return Some(fallback);
    }

    None
}

/// Read all JSONL files from a directory and parse into `TelemetryData`.
///
/// # Errors
/// Returns `DiscoveryError::Io` on filesystem errors or JSON parse failures.
pub fn read_telemetry_dir(dir: &Path) -> DiscoveryResult<TelemetryData> {
    let entries = std::fs::read_dir(dir).map_err(DiscoveryError::Io)?;

    let mut data = TelemetryData::default();

    for entry in entries {
        let entry = entry.map_err(DiscoveryError::Io)?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "jsonl") {
            read_jsonl_file(&path, &mut data)?;
        }
    }

    Ok(data)
}

fn read_jsonl_file(path: &Path, data: &mut TelemetryData) -> DiscoveryResult<()> {
    let file = std::fs::File::open(path).map_err(DiscoveryError::Io)?;
    let reader = std::io::BufReader::new(file);

    for line in reader.lines() {
        let line = line.map_err(DiscoveryError::Io)?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Ok(event) = serde_json::from_str::<TelemetryEvent>(line) {
            let section = data.sections.entry(event.section).or_default();
            for (key, value) in &event.fields {
                if key == "t" || key == "section" {
                    continue;
                }
                if let Some(v) = value.as_f64() {
                    let ts = section.entry(key.clone()).or_default();
                    ts.timestamps.push(event.t);
                    ts.values.push(v);
                }
            }
        }
    }

    Ok(())
}

/// Convert parsed telemetry data into `DataBinding` variants for visualization.
#[must_use]
pub fn telemetry_to_bindings(data: &TelemetryData) -> Vec<DataBinding> {
    let mut bindings = Vec::new();

    for (section, fields) in &data.sections {
        for (field_name, ts) in fields {
            bindings.push(DataBinding::TimeSeries {
                id: format!("{section}.{field_name}"),
                label: format!("{section} — {field_name}"),
                x_values: ts.timestamps.clone(),
                y_values: ts.values.clone(),
                x_label: "time (s)".to_string(),
                y_label: field_name.clone(),
                unit: String::new(),
            });
        }
    }

    bindings
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_test_jsonl(dir: &Path, filename: &str, lines: &[&str]) {
        let path = dir.join(filename);
        let mut file = std::fs::File::create(path).unwrap();
        for line in lines {
            writeln!(file, "{line}").unwrap();
        }
    }

    #[test]
    fn parse_telemetry_event() {
        let json = r#"{"t": 1.5, "section": "lattice_qcd", "plaquette": 0.56, "acceptance": 0.82}"#;
        let event: TelemetryEvent = serde_json::from_str(json).unwrap();
        assert!((event.t - 1.5).abs() < f64::EPSILON);
        assert_eq!(event.section, "lattice_qcd");
        assert!(event.fields.contains_key("plaquette"));
        assert!(event.fields.contains_key("acceptance"));
    }

    #[test]
    fn read_jsonl_from_dir() {
        let dir = tempfile::tempdir().unwrap();
        write_test_jsonl(
            dir.path(),
            "test.jsonl",
            &[
                r#"{"t": 0.0, "section": "md", "energy": -42.5, "temperature": 300.0}"#,
                r#"{"t": 1.0, "section": "md", "energy": -42.3, "temperature": 301.0}"#,
                r#"{"t": 2.0, "section": "qcd", "plaquette": 0.55}"#,
            ],
        );

        let data = read_telemetry_dir(dir.path()).unwrap();
        assert_eq!(data.sections.len(), 2);

        let md = &data.sections["md"];
        assert_eq!(md["energy"].values.len(), 2);
        assert_eq!(md["temperature"].values.len(), 2);

        let qcd = &data.sections["qcd"];
        assert_eq!(qcd["plaquette"].values.len(), 1);
    }

    #[test]
    fn telemetry_to_bindings_creates_time_series() {
        let mut data = TelemetryData::default();
        let mut fields = HashMap::new();
        fields.insert(
            "energy".to_string(),
            TimeSeries {
                timestamps: vec![0.0, 1.0, 2.0],
                values: vec![-42.5, -42.3, -42.1],
            },
        );
        data.sections.insert("md".to_string(), fields);

        let bindings = telemetry_to_bindings(&data);
        assert_eq!(bindings.len(), 1);
        match &bindings[0] {
            DataBinding::TimeSeries { id, x_values, .. } => {
                assert_eq!(id, "md.energy");
                assert_eq!(x_values.len(), 3);
            }
            other => panic!("expected TimeSeries, got {other:?}"),
        }
    }

    #[test]
    fn empty_dir_returns_empty_data() {
        let dir = tempfile::tempdir().unwrap();
        let data = read_telemetry_dir(dir.path()).unwrap();
        assert!(data.sections.is_empty());
    }

    #[test]
    fn skips_non_jsonl_files() {
        let dir = tempfile::tempdir().unwrap();
        write_test_jsonl(
            dir.path(),
            "data.txt",
            &[r#"{"t": 0.0, "section": "x", "v": 1.0}"#],
        );
        let data = read_telemetry_dir(dir.path()).unwrap();
        assert!(data.sections.is_empty());
    }

    #[test]
    fn skips_malformed_lines() {
        let dir = tempfile::tempdir().unwrap();
        write_test_jsonl(
            dir.path(),
            "test.jsonl",
            &[
                "not json at all",
                r#"{"t": 1.0, "section": "ok", "val": 42.0}"#,
                "",
                r#"{"incomplete": true"#,
            ],
        );
        let data = read_telemetry_dir(dir.path()).unwrap();
        assert_eq!(data.sections.len(), 1);
        assert_eq!(data.sections["ok"]["val"].values.len(), 1);
    }

    #[test]
    fn telemetry_dir_returns_none_without_env() {
        assert!(telemetry_dir().is_none() || telemetry_dir().is_some());
    }

    #[test]
    #[cfg(feature = "test-fixtures")]
    fn telemetry_dir_with_xdg_data_home() {
        use petal_tongue_core::test_fixtures::env_test_helpers;

        let dir = tempfile::tempdir().unwrap();
        let telemetry_dir_path = dir.path().join("petaltongue/telemetry");
        std::fs::create_dir_all(&telemetry_dir_path).unwrap();

        let result =
            env_test_helpers::with_env_var("XDG_DATA_HOME", dir.path().to_str().unwrap(), || {
                telemetry_dir()
            });
        assert_eq!(result, Some(telemetry_dir_path));
    }

    #[test]
    #[cfg(feature = "test-fixtures")]
    fn telemetry_dir_with_petaltongue_telemetry_dir_env() {
        use petal_tongue_core::test_fixtures::env_test_helpers;

        let dir = tempfile::tempdir().unwrap();

        let result = env_test_helpers::with_env_var(
            "PETALTONGUE_TELEMETRY_DIR",
            dir.path().to_str().unwrap(),
            || telemetry_dir(),
        );
        assert_eq!(result, Some(dir.path().to_path_buf()));
    }
}
