// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests that all sandbox scenario JSON files are valid JSON and can be loaded.

use petal_tongue_core::scenario_loader::LoadedScenario;
use std::path::Path;

#[test]
fn all_sandbox_scenarios_are_valid_json() {
    let scenarios_dir = format!("{}/../../sandbox/scenarios", env!("CARGO_MANIFEST_DIR"));
    let path = Path::new(&scenarios_dir);
    let entries = std::fs::read_dir(path).expect("sandbox/scenarios directory must exist");

    let mut count = 0;
    let mut failed = Vec::new();

    for entry in entries {
        let entry = entry.expect("read_dir entry");
        let file_path = entry.path();
        if file_path.extension().is_some_and(|ext| ext == "json") {
            count += 1;
            let file_name = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();

            let contents = std::fs::read_to_string(&file_path)
                .unwrap_or_else(|e| panic!("Failed to read {file_name}: {e}"));
            if let Err(e) = serde_json::from_str::<serde_json::Value>(&contents) {
                failed.push((file_name, format!("Invalid JSON: {e}")));
            }
        }
    }

    assert!(count > 0, "No scenario files found");
    assert!(
        failed.is_empty(),
        "{} file(s) are not valid JSON:\n{:#?}",
        failed.len(),
        failed
    );
}

#[test]
fn ecosystem_scenario_files_parse_as_loaded_scenario() {
    let scenarios_dir = format!("{}/../../sandbox/scenarios", env!("CARGO_MANIFEST_DIR"));
    let path = Path::new(&scenarios_dir);
    let entries = std::fs::read_dir(path).expect("sandbox/scenarios directory must exist");

    let mut parsed = 0;
    let mut total = 0;

    for entry in entries {
        let entry = entry.expect("read_dir entry");
        let file_path = entry.path();
        if file_path.extension().is_some_and(|ext| ext == "json") {
            total += 1;
            let file_name = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();

            let contents = std::fs::read_to_string(&file_path).unwrap();
            let json: serde_json::Value = serde_json::from_str(&contents).unwrap();
            let has_ecosystem = json.get("ecosystem").is_some();

            if has_ecosystem {
                match LoadedScenario::from_file(&file_path) {
                    Ok(scenario) => {
                        parsed += 1;
                        let binding_count = scenario.all_bindings().len();
                        eprintln!(
                            "  OK: {file_name} ({} primals, {binding_count} bindings)",
                            scenario.ecosystem.primals.len()
                        );
                    }
                    Err(e) => {
                        panic!("Ecosystem scenario {file_name} should parse but failed: {e}");
                    }
                }
            }
        }
    }

    assert!(
        total >= 20,
        "Expected at least 20 scenario files, found {total}"
    );
    assert!(
        parsed >= 10,
        "Expected at least 10 ecosystem scenarios, found {parsed}"
    );
}
