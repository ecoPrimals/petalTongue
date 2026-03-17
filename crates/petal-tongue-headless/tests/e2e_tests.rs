// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! End-to-end tests for petal-tongue-headless binary

#![expect(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;

/// Get path to the headless binary
fn headless_binary() -> Command {
    Command::cargo_bin("petal-tongue-headless").expect("Failed to find binary")
}

#[test]
fn test_help_flag() {
    headless_binary()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("petalTongue Headless"))
        .stdout(predicate::str::contains("Pure Rust UI"))
        .stdout(predicate::str::contains("--mode"))
        .stdout(predicate::str::contains("--output"));
}

#[test]
fn test_terminal_mode() {
    headless_binary()
        .arg("--mode")
        .arg("terminal")
        .env("RUST_LOG", "error") // Suppress logs for cleaner output
        .assert()
        .success()
        .stdout(predicate::str::contains("petalTongue Topology"))
        .stdout(predicate::str::contains("PRIMALS:"))
        .stdout(predicate::str::contains("CONNECTIONS:"));
}

#[test]
fn test_svg_export_to_stdout() {
    headless_binary()
        .arg("--mode")
        .arg("svg")
        .env("RUST_LOG", "error")
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("</svg>"));
}

#[test]
fn test_json_export_to_stdout() {
    headless_binary()
        .arg("--mode")
        .arg("json")
        .env("RUST_LOG", "error")
        .assert()
        .success()
        .stdout(predicate::str::contains("topology"))
        .stdout(predicate::str::contains("primals"))
        .stdout(predicate::str::contains("connections"));
}

#[test]
fn test_dot_export_to_stdout() {
    headless_binary()
        .arg("--mode")
        .arg("dot")
        .env("RUST_LOG", "error")
        .assert()
        .success()
        .stdout(predicate::str::contains("digraph PetalTongue"))
        .stdout(predicate::str::contains("->"));
}

#[test]
fn test_svg_export_to_file() {
    let temp_dir = std::env::temp_dir();
    let output_file = temp_dir.join("e2e_test_topology.svg");

    // Remove file if it exists
    let _ = fs::remove_file(&output_file);

    headless_binary()
        .arg("--mode")
        .arg("svg")
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .env("RUST_LOG", "error")
        .assert()
        .success();

    // Verify file was created
    assert!(output_file.exists());

    // Verify content
    let content = fs::read_to_string(&output_file).expect("Failed to read file");
    assert!(content.contains("<svg"));
    assert!(content.contains("</svg>"));

    // Cleanup
    fs::remove_file(output_file).ok();
}

#[test]
fn test_json_export_to_file() {
    let temp_dir = std::env::temp_dir();
    let output_file = temp_dir.join("e2e_test_topology.json");

    // Remove file if it exists
    let _ = fs::remove_file(&output_file);

    headless_binary()
        .arg("--mode")
        .arg("json")
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .env("RUST_LOG", "error")
        .assert()
        .success();

    // Verify file was created
    assert!(output_file.exists());

    // Verify content is valid JSON
    let content = fs::read_to_string(&output_file).expect("Failed to read file");
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON");
    assert!(parsed["topology"].is_object());
    assert!(parsed["topology"]["primals"].is_array());

    // Cleanup
    fs::remove_file(output_file).ok();
}

#[test]
fn test_dot_export_to_file() {
    let temp_dir = std::env::temp_dir();
    let output_file = temp_dir.join("e2e_test_topology.dot");

    // Remove file if it exists
    let _ = fs::remove_file(&output_file);

    headless_binary()
        .arg("--mode")
        .arg("dot")
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .env("RUST_LOG", "error")
        .assert()
        .success();

    // Verify file was created
    assert!(output_file.exists());

    // Verify content
    let content = fs::read_to_string(&output_file).expect("Failed to read file");
    assert!(content.contains("digraph"));

    // Cleanup
    fs::remove_file(output_file).ok();
}

#[test]
fn test_custom_dimensions() {
    headless_binary()
        .arg("--mode")
        .arg("svg")
        .arg("--width")
        .arg("3840")
        .arg("--height")
        .arg("2160")
        .env("RUST_LOG", "error")
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("3840"))
        .stdout(predicate::str::contains("2160"));
}

#[test]
fn test_invalid_mode() {
    headless_binary()
        .arg("--mode")
        .arg("invalid")
        .assert()
        .failure()
        .stdout(predicate::str::contains("Unknown mode"));
}

#[test]
fn test_unknown_argument() {
    headless_binary()
        .arg("--unknown")
        .assert()
        .failure()
        .stdout(predicate::str::contains("Unknown argument"));
}

#[test]
fn test_png_without_output() {
    headless_binary()
        .arg("--mode")
        .arg("png")
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "PNG mode requires --output option",
        ));
}

#[test]
fn test_auto_mode() {
    // Auto mode should work without crashing
    headless_binary()
        .env("RUST_LOG", "error")
        .assert()
        .success();
}

// Logging test removed - output format varies by environment

#[test]
fn test_showcase_mode() {
    headless_binary()
        .arg("--mode")
        .arg("terminal")
        .env("SHOWCASE_MODE", "true")
        .env("RUST_LOG", "error")
        .assert()
        .success()
        .stdout(predicate::str::contains("petalTongue Topology"));
}

#[test]
fn test_output_consistency() {
    let temp_dir = std::env::temp_dir();

    // Generate same topology twice
    let file1 = temp_dir.join("consistency_test_1.json");
    let file2 = temp_dir.join("consistency_test_2.json");

    let _ = fs::remove_file(&file1);
    let _ = fs::remove_file(&file2);

    // First run
    headless_binary()
        .arg("--mode")
        .arg("json")
        .arg("--output")
        .arg(file1.to_str().unwrap())
        .env("RUST_LOG", "error")
        .assert()
        .success();

    // Second run
    headless_binary()
        .arg("--mode")
        .arg("json")
        .arg("--output")
        .arg(file2.to_str().unwrap())
        .env("RUST_LOG", "error")
        .assert()
        .success();

    // Compare (should be identical topology structure)
    let content1 = fs::read_to_string(&file1).expect("Failed to read file1");
    let content2 = fs::read_to_string(&file2).expect("Failed to read file2");

    let json1: serde_json::Value = serde_json::from_str(&content1).unwrap();
    let json2: serde_json::Value = serde_json::from_str(&content2).unwrap();

    // Same number of primals and connections
    assert_eq!(
        json1["topology"]["primals"].as_array().unwrap().len(),
        json2["topology"]["primals"].as_array().unwrap().len()
    );
    assert_eq!(
        json1["topology"]["connections"].as_array().unwrap().len(),
        json2["topology"]["connections"].as_array().unwrap().len()
    );

    // Cleanup
    fs::remove_file(file1).ok();
    fs::remove_file(file2).ok();
}

#[test]
fn test_concurrent_exports() {
    use std::thread;

    let temp_dir = std::env::temp_dir();
    let files: Vec<PathBuf> = (0..5)
        .map(|i| temp_dir.join(format!("concurrent_test_{i}.svg")))
        .collect();

    // Remove existing files
    for file in &files {
        let _ = fs::remove_file(file);
    }

    // Spawn multiple concurrent exports
    let handles: Vec<_> = files
        .iter()
        .map(|file| {
            let file = file.clone();
            thread::spawn(move || {
                headless_binary()
                    .arg("--mode")
                    .arg("svg")
                    .arg("--output")
                    .arg(file.to_str().unwrap())
                    .env("RUST_LOG", "error")
                    .assert()
                    .success();
            })
        })
        .collect();

    // Wait for all to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify all files were created
    for file in &files {
        assert!(file.exists());
        let content = fs::read_to_string(file).expect("Failed to read file");
        assert!(content.contains("<svg"));
    }

    // Cleanup
    for file in files {
        fs::remove_file(file).ok();
    }
}
