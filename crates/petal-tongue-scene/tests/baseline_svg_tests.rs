// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::expect_used, reason = "test assertions")]

//! Tier 2 SVG validation: compile representative DataBindings through the
//! full primal pipeline (DataBindingCompiler → GrammarCompiler → SvgCompiler)
//! and assert structural SVG validity.
//!
//! These tests are hermetic — all data is inline, no cross-workspace deps.

use petal_tongue_scene::compiler::GrammarCompiler;
use petal_tongue_scene::data_binding::DataBindingCompiler;
use petal_tongue_scene::modality::{ModalityCompiler, ModalityOutput, SvgCompiler};
use petal_tongue_types::DataBinding;
use serde_json::json;

fn compile_to_svg(binding: &DataBinding, domain: Option<&str>) -> String {
    let (expr, data) = DataBindingCompiler::compile(binding, domain);
    let compiler = GrammarCompiler::new();
    let scene = compiler.compile(&expr, &data);
    let svg_compiler = SvgCompiler::new();
    let output = svg_compiler.compile(&scene);
    match output {
        ModalityOutput::Svg(bytes) => String::from_utf8(bytes.to_vec()).expect("valid UTF-8"),
        other => panic!("expected SVG output, got {other:?}"),
    }
}

fn assert_valid_svg(svg: &str, label: &str) {
    assert!(
        svg.starts_with("<svg"),
        "{label}: SVG should start with <svg, starts with {:?}",
        &svg[..svg.len().min(40)]
    );
    assert!(
        svg.ends_with("</svg>"),
        "{label}: SVG should end with </svg>"
    );
    assert!(
        !svg.contains("NaN"),
        "{label}: SVG should not contain NaN coordinates"
    );
    assert!(
        !svg.contains("Infinity"),
        "{label}: SVG should not contain Infinity coordinates"
    );
}

fn assert_contains_element(svg: &str, element: &str, label: &str) {
    assert!(
        svg.contains(element),
        "{label}: SVG should contain {element}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// Baseline DataBinding factories (hermetic, inline data)
// ──────────────────────────────────────────────────────────────────────────

fn breseq_genome_track() -> DataBinding {
    DataBinding::GenomeTrack {
        id: "bl_breseq_genome".into(),
        label: "breseq Genome Track".into(),
        sequence_length: 4_629_812.0,
        tracks: vec!["SNP".into(), "IS Element".into(), "Large Deletion".into()],
        segments: vec![
            json!({"track": "SNP", "start": 70867.0, "end": 70868.0, "strand": ".", "label": "SNP"}),
            json!({"track": "SNP", "start": 1_234_567.0, "end": 1_234_568.0, "strand": ".", "label": "SNP2"}),
            json!({"track": "IS Element", "start": 776_697.0, "end": 778_028.0, "strand": "+", "label": "IS1"}),
            json!({"track": "Large Deletion", "start": 1_298_712.0, "end": 1_330_044.0, "strand": ".", "label": "DEL1"}),
        ],
        unit: "bp".into(),
    }
}

fn breseq_evidence_bar() -> DataBinding {
    DataBinding::Bar {
        id: "bl_breseq_evidence_types".into(),
        label: "breseq Evidence Types".into(),
        categories: vec!["RA".into(), "MC".into(), "JC".into(), "UN".into()],
        values: vec![42.0, 12.0, 23.0, 17.0],
        unit: "count".into(),
    }
}

fn breseq_mutations_gauge() -> DataBinding {
    DataBinding::Gauge {
        id: "bl_breseq_total_mutations".into(),
        label: "Total Predicted Mutations".into(),
        value: 94.0,
        min: 0.0,
        max: 200.0,
        unit: "mutations".into(),
        normal_range: [0.0, 50.0],
        warning_range: [50.0, 150.0],
    }
}

fn plannotate_circular_map() -> DataBinding {
    DataBinding::CircularMap {
        id: "bl_plannotate_map".into(),
        label: "pUC19 Circular Map".into(),
        sequence_length: 2686.0,
        rings: vec!["features".into()],
        arcs: vec![
            json!({"start_angle": 0.0, "end_angle": 90.0, "ring": 0, "label": "ori"}),
            json!({"start_angle": 120.0, "end_angle": 200.0, "ring": 0, "label": "AmpR"}),
            json!({"start_angle": 240.0, "end_angle": 280.0, "ring": 0, "label": "lacZ"}),
        ],
        unit: "bp".into(),
    }
}

fn plannotate_features_bar() -> DataBinding {
    DataBinding::Bar {
        id: "bl_plannotate_features".into(),
        label: "Feature Annotation Lengths".into(),
        categories: vec!["ori".into(), "AmpR".into(), "lacZ".into()],
        values: vec![600.0, 860.0, 510.0],
        unit: "bp".into(),
    }
}

fn ostir_scatter() -> DataBinding {
    DataBinding::Scatter {
        id: "bl_ostir_tir_scatter".into(),
        label: "OSTIR TIR Predictions".into(),
        x: vec![42.0, 156.0, 891.0, 12345.0, 72891.0],
        y: vec![1200.0, 45000.0, 8900.0, 120.0, 67000.0],
        point_labels: vec![
            "RBS1".into(),
            "RBS2".into(),
            "RBS3".into(),
            "RBS4".into(),
            "RBS5".into(),
        ],
        x_label: "Position (nt)".into(),
        y_label: "TIR (au)".into(),
        unit: "au".into(),
    }
}

fn ostir_distribution() -> DataBinding {
    DataBinding::Distribution {
        id: "bl_ostir_rate_dist".into(),
        label: "TIR Rate Distribution".into(),
        values: vec![42.8, 127.5, 85.2, 43.1, 99.8, 150.3, 67.4],
        mean: 85.15,
        std: 59.9,
        comparison_value: 100.0,
        unit: "au".into(),
    }
}

fn cryptkeeper_genome_track() -> DataBinding {
    DataBinding::GenomeTrack {
        id: "bl_cryptkeeper_track".into(),
        label: "CryptKeeper Promoter Analysis".into(),
        sequence_length: 4_629_812.0,
        tracks: vec!["ORFs/Features".into(), "Cryptic Promoters".into()],
        segments: vec![
            json!({"track": "ORFs/Features", "start": 100_000.0, "end": 102_000.0, "strand": "+", "label": "lacZ"}),
            json!({"track": "ORFs/Features", "start": 500_000.0, "end": 501_500.0, "strand": "-", "label": "araC"}),
            json!({"track": "Cryptic Promoters", "start": 101_800.0, "end": 102_200.0, "strand": "+", "label": "P_crypto_1"}),
        ],
        unit: "bp".into(),
    }
}

fn efm_genome_track() -> DataBinding {
    DataBinding::GenomeTrack {
        id: "bl_efm_rate_track".into(),
        label: "EFM Rate-Colored Features".into(),
        sequence_length: 4_629_812.0,
        tracks: vec![
            "IS Target".into(),
            "Repeat Indel".into(),
            "Base Sub Hotspot".into(),
        ],
        segments: vec![
            json!({"track": "IS Target", "start": 776_697.0, "end": 778_028.0, "strand": "+", "label": "IS1"}),
            json!({"track": "Repeat Indel", "start": 1_200_000.0, "end": 1_200_500.0, "strand": ".", "label": "repeat1"}),
            json!({"track": "Base Sub Hotspot", "start": 3_500_000.0, "end": 3_500_100.0, "strand": ".", "label": "hotspot1"}),
        ],
        unit: "bp".into(),
    }
}

fn marker_divergence_scatter() -> DataBinding {
    DataBinding::Scatter {
        id: "bl_md_divergence_exp_1".into(),
        label: "Marker Divergence Experiment 1".into(),
        x: vec![0.0, 100.0, 200.0, 500.0, 1000.0],
        y: vec![0.0, 0.15, 0.28, 0.52, 0.78],
        point_labels: vec![
            "t0".into(),
            "t100".into(),
            "t200".into(),
            "t500".into(),
            "t1000".into(),
        ],
        x_label: "Generations".into(),
        y_label: "Divergence".into(),
        unit: "rel".into(),
    }
}

fn rna_mi_heatmap() -> DataBinding {
    DataBinding::Heatmap {
        id: "bl_rna_mi_covariance".into(),
        label: "RNA Mutual Information".into(),
        x_labels: vec!["pos1".into(), "pos2".into(), "pos3".into(), "pos4".into()],
        y_labels: vec!["pos1".into(), "pos2".into(), "pos3".into(), "pos4".into()],
        values: vec![
            1.0, 0.8, 0.2, 0.1, 0.8, 1.0, 0.3, 0.15, 0.2, 0.3, 1.0, 0.7, 0.1, 0.15, 0.7, 1.0,
        ],
        unit: "bits".into(),
    }
}

fn rna_mi_spectrum() -> DataBinding {
    DataBinding::Spectrum {
        id: "bl_rna_mi_entropy".into(),
        label: "Positional Entropy".into(),
        frequencies: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
        amplitudes: vec![0.3, 0.7, 0.5, 0.9, 0.2, 0.6, 0.4, 0.8],
        unit: "bits".into(),
    }
}

fn timeseries_binding() -> DataBinding {
    DataBinding::TimeSeries {
        id: "bl_ts".into(),
        label: "Growth Curve".into(),
        x_label: "Time (h)".into(),
        y_label: "OD600".into(),
        x_values: vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0],
        y_values: vec![0.05, 0.12, 0.35, 0.78, 1.2, 1.5],
        unit: "OD".into(),
    }
}

fn all_baseline_bindings() -> Vec<(&'static str, DataBinding)> {
    vec![
        ("breseq_genome_track", breseq_genome_track()),
        ("breseq_evidence_bar", breseq_evidence_bar()),
        ("breseq_mutations_gauge", breseq_mutations_gauge()),
        ("plannotate_circular_map", plannotate_circular_map()),
        ("plannotate_features_bar", plannotate_features_bar()),
        ("ostir_scatter", ostir_scatter()),
        ("ostir_distribution", ostir_distribution()),
        ("cryptkeeper_genome_track", cryptkeeper_genome_track()),
        ("efm_genome_track", efm_genome_track()),
        ("marker_divergence_scatter", marker_divergence_scatter()),
        ("rna_mi_heatmap", rna_mi_heatmap()),
        ("rna_mi_spectrum", rna_mi_spectrum()),
        ("timeseries", timeseries_binding()),
    ]
}

// ──────────────────────────────────────────────────────────────────────────
// SVG structural validation tests
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn all_baselines_compile_to_valid_svg() {
    for (name, binding) in all_baseline_bindings() {
        let svg = compile_to_svg(&binding, Some("genomics"));
        assert_valid_svg(&svg, name);
        assert!(
            svg.len() > 100,
            "{name}: SVG should be substantial, got {len} bytes",
            len = svg.len()
        );
    }
}

#[test]
fn genome_track_svg_contains_rects() {
    let svg = compile_to_svg(&breseq_genome_track(), Some("genomics"));
    assert_valid_svg(&svg, "breseq_genome_track");
    assert_contains_element(&svg, "<rect", "breseq_genome_track");
}

#[test]
fn circular_map_svg_contains_polygons() {
    let svg = compile_to_svg(&plannotate_circular_map(), Some("genomics"));
    assert_valid_svg(&svg, "plannotate_circular_map");
    assert_contains_element(&svg, "<polygon", "plannotate_circular_map");
}

#[test]
fn bar_chart_svg_contains_rects() {
    let svg = compile_to_svg(&breseq_evidence_bar(), None);
    assert_valid_svg(&svg, "breseq_evidence_bar");
    assert_contains_element(&svg, "<rect", "breseq_evidence_bar");
}

#[test]
fn gauge_svg_contains_arc_path() {
    let svg = compile_to_svg(&breseq_mutations_gauge(), None);
    assert_valid_svg(&svg, "breseq_mutations_gauge");
    assert_contains_element(&svg, "<path", "breseq_mutations_gauge");
}

#[test]
fn scatter_svg_contains_circles() {
    let svg = compile_to_svg(&ostir_scatter(), None);
    assert_valid_svg(&svg, "ostir_scatter");
    assert_contains_element(&svg, "<circle", "ostir_scatter");
}

#[test]
fn heatmap_svg_contains_rects() {
    let svg = compile_to_svg(&rna_mi_heatmap(), None);
    assert_valid_svg(&svg, "rna_mi_heatmap");
    assert_contains_element(&svg, "<rect", "rna_mi_heatmap");
}

#[test]
fn timeseries_svg_contains_polyline() {
    let svg = compile_to_svg(&timeseries_binding(), None);
    assert_valid_svg(&svg, "timeseries");
    assert_contains_element(&svg, "<polyline", "timeseries");
}

#[test]
fn svg_contains_text_labels() {
    let svg = compile_to_svg(&breseq_genome_track(), Some("genomics"));
    assert_contains_element(&svg, "<text", "breseq_genome_track");
}

#[test]
fn multiple_bindings_produce_distinct_svgs() {
    let bindings = all_baseline_bindings();
    let svgs: Vec<String> = bindings
        .iter()
        .map(|(_, b)| compile_to_svg(b, Some("genomics")))
        .collect();

    for i in 0..svgs.len() {
        for j in (i + 1)..svgs.len() {
            assert_ne!(
                svgs[i], svgs[j],
                "SVGs for {} and {} should be distinct",
                bindings[i].0, bindings[j].0
            );
        }
    }
}

#[test]
fn svg_viewbox_is_present() {
    for (name, binding) in all_baseline_bindings() {
        let svg = compile_to_svg(&binding, None);
        assert!(
            svg.contains("viewBox"),
            "{name}: SVG should have a viewBox attribute"
        );
    }
}
