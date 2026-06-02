// SPDX-License-Identifier: AGPL-3.0-or-later
//! Baseline DataBinding export for Barrick Lab tools validation.

use crate::args::Args;
use crate::error::HeadlessError;
use std::path::Path;

/// Export all baseline DataBindings as individual SVGs for human review.
pub fn render_baselines(args: &Args) -> Result<(), HeadlessError> {
    use petal_tongue_scene::compiler::GrammarCompiler;
    use petal_tongue_scene::data_binding::DataBindingCompiler;
    use petal_tongue_scene::modality::{ModalityCompiler, ModalityOutput, SvgCompiler};

    let output_dir = args.output.as_deref().unwrap_or("baselines_svg");
    let out_path = Path::new(output_dir);
    std::fs::create_dir_all(out_path)?;

    let bindings = baseline_bindings();
    let compiler = GrammarCompiler::new();
    let svg_compiler = SvgCompiler::new();

    let mut success_count = 0u32;
    let mut error_count = 0u32;

    for (name, binding) in &bindings {
        let (expr, data) = DataBindingCompiler::compile(binding, Some("genomics"));
        let scene = compiler.compile(&expr, &data);

        if scene.total_primitives() == 0 {
            tracing::warn!("  {name}: empty scene (0 primitives), skipping");
            error_count += 1;
            continue;
        }

        let output = svg_compiler.compile(&scene);
        let ModalityOutput::Svg(svg_bytes) = output else {
            tracing::warn!("  {name}: unexpected modality output");
            error_count += 1;
            continue;
        };

        let file_name = format!("{name}.svg");
        let file_path = out_path.join(&file_name);
        std::fs::write(&file_path, &svg_bytes)?;

        tracing::info!(
            "  {name}: {} primitives -> {}",
            scene.total_primitives(),
            file_path.display()
        );
        success_count += 1;
    }

    println!(
        "Baselines export: {success_count} SVGs exported, {error_count} errors -> {}",
        out_path.display()
    );

    Ok(())
}

/// All baseline DataBindings for Barrick Lab tools validation.
#[expect(
    clippy::too_many_lines,
    reason = "headless baseline catalog: many representative DataBinding fixtures in one table"
)]
fn baseline_bindings() -> Vec<(&'static str, petal_tongue_core::DataBinding)> {
    use petal_tongue_core::DataBinding;
    use serde_json::json;

    vec![
        (
            "breseq_genome_track",
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
            },
        ),
        (
            "breseq_evidence_bar",
            DataBinding::Bar {
                id: "bl_breseq_evidence".into(),
                label: "Evidence Types".into(),
                categories: vec!["RA".into(), "MC".into(), "JC".into(), "UN".into()],
                values: vec![42.0, 12.0, 23.0, 17.0],
                unit: "count".into(),
            },
        ),
        (
            "breseq_mutations_gauge",
            DataBinding::Gauge {
                id: "bl_breseq_mutations".into(),
                label: "Total Mutations".into(),
                value: 94.0,
                min: 0.0,
                max: 200.0,
                unit: "mutations".into(),
                normal_range: [0.0, 50.0],
                warning_range: [50.0, 150.0],
            },
        ),
        (
            "breseq_coverage_ts",
            DataBinding::TimeSeries {
                id: "bl_breseq_coverage".into(),
                label: "Read Coverage".into(),
                x_label: "Position (bp)".into(),
                y_label: "Coverage".into(),
                x_values: vec![
                    0.0,
                    500_000.0,
                    1_000_000.0,
                    2_000_000.0,
                    3_000_000.0,
                    4_000_000.0,
                ],
                y_values: vec![45.0, 52.0, 48.0, 55.0, 43.0, 50.0],
                unit: "reads".into(),
            },
        ),
        (
            "plannotate_circular_map",
            DataBinding::CircularMap {
                id: "bl_plannotate_map".into(),
                label: "pUC19".into(),
                sequence_length: 2686.0,
                rings: vec!["features".into()],
                arcs: vec![
                    json!({"start_angle": 0.0, "end_angle": 90.0, "ring": 0, "label": "ori"}),
                    json!({"start_angle": 120.0, "end_angle": 200.0, "ring": 0, "label": "AmpR"}),
                    json!({"start_angle": 240.0, "end_angle": 280.0, "ring": 0, "label": "lacZ"}),
                ],
                unit: "bp".into(),
            },
        ),
        (
            "plannotate_features_bar",
            DataBinding::Bar {
                id: "bl_plannotate_features".into(),
                label: "Feature Lengths".into(),
                categories: vec!["ori".into(), "AmpR".into(), "lacZ".into()],
                values: vec![600.0, 860.0, 510.0],
                unit: "bp".into(),
            },
        ),
        (
            "plannotate_confidence",
            DataBinding::Scatter {
                id: "bl_plannotate_conf".into(),
                label: "Annotation Confidence".into(),
                x: vec![600.0, 860.0, 510.0],
                y: vec![0.99, 0.95, 0.87],
                point_labels: vec!["ori".into(), "AmpR".into(), "lacZ".into()],
                x_label: "Length (bp)".into(),
                y_label: "Confidence".into(),
                unit: String::new(),
            },
        ),
        (
            "ostir_tir_scatter",
            DataBinding::Scatter {
                id: "bl_ostir_tir".into(),
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
            },
        ),
        (
            "ostir_rate_dist",
            DataBinding::Distribution {
                id: "bl_ostir_rate_dist".into(),
                label: "TIR Distribution".into(),
                values: vec![42.8, 127.5, 85.2, 43.1, 99.8, 150.3, 67.4],
                mean: 85.15,
                std: 59.9,
                comparison_value: 100.0,
                unit: "au".into(),
            },
        ),
        (
            "ostir_energy_bar",
            DataBinding::Bar {
                id: "bl_ostir_energy".into(),
                label: "dG Decomposition".into(),
                categories: vec![
                    "dG_total".into(),
                    "dG_mRNA".into(),
                    "dG_spacing".into(),
                    "dG_standby".into(),
                ],
                values: vec![-8.2, -3.1, -2.8, -2.3],
                unit: "kcal/mol".into(),
            },
        ),
        (
            "cryptkeeper_track",
            DataBinding::GenomeTrack {
                id: "bl_cryptkeeper".into(),
                label: "CryptKeeper Analysis".into(),
                sequence_length: 4_629_812.0,
                tracks: vec!["ORFs/Features".into(), "Cryptic Promoters".into()],
                segments: vec![
                    json!({"track": "ORFs/Features", "start": 100_000.0, "end": 102_000.0, "strand": "+", "label": "lacZ"}),
                    json!({"track": "ORFs/Features", "start": 500_000.0, "end": 501_500.0, "strand": "-", "label": "araC"}),
                    json!({"track": "Cryptic Promoters", "start": 101_800.0, "end": 102_200.0, "strand": "+", "label": "P_crypto_1"}),
                    json!({"track": "Cryptic Promoters", "start": 800_000.0, "end": 800_400.0, "strand": "+", "label": "P_crypto_2"}),
                ],
                unit: "bp".into(),
            },
        ),
        (
            "cryptkeeper_burden_heatmap",
            DataBinding::Heatmap {
                id: "bl_cryptkeeper_burden".into(),
                label: "Promoter Burden".into(),
                x_labels: vec!["lacZ".into(), "araC".into(), "galK".into()],
                y_labels: vec!["strong".into(), "medium".into(), "weak".into()],
                values: vec![0.9, 0.3, 0.5, 0.6, 0.7, 0.4, 0.2, 0.8, 0.1],
                unit: "burden".into(),
            },
        ),
        (
            "efm_rate_track",
            DataBinding::GenomeTrack {
                id: "bl_efm_track".into(),
                label: "EFM Rate-Colored Features".into(),
                sequence_length: 4_629_812.0,
                tracks: vec![
                    "IS Target".into(),
                    "Repeat Indel".into(),
                    "Base Sub Hotspot".into(),
                ],
                segments: vec![
                    json!({"track": "IS Target", "start": 776_697.0, "end": 778_028.0, "strand": "+", "label": "IS1"}),
                    json!({"track": "IS Target", "start": 1_500_000.0, "end": 1_501_200.0, "strand": "+", "label": "IS5"}),
                    json!({"track": "Repeat Indel", "start": 1_200_000.0, "end": 1_200_500.0, "strand": ".", "label": "repeat1"}),
                    json!({"track": "Base Sub Hotspot", "start": 3_500_000.0, "end": 3_500_100.0, "strand": ".", "label": "hotspot1"}),
                ],
                unit: "bp".into(),
            },
        ),
        (
            "efm_rate_bar",
            DataBinding::Bar {
                id: "bl_efm_rate_bar".into(),
                label: "Mutation Rates by Category".into(),
                categories: vec![
                    "IS Insertion".into(),
                    "Repeat Indel".into(),
                    "Base Sub".into(),
                ],
                values: vec![2.3e-6, 8.7e-7, 1.1e-9],
                unit: "per bp per gen".into(),
            },
        ),
        (
            "md_divergence_scatter",
            DataBinding::Scatter {
                id: "bl_md_divergence".into(),
                label: "Marker Divergence".into(),
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
            },
        ),
        (
            "md_trajectory_ts",
            DataBinding::TimeSeries {
                id: "bl_md_trajectory".into(),
                label: "Population Trajectory".into(),
                x_label: "Generations".into(),
                y_label: "Frequency".into(),
                x_values: vec![0.0, 100.0, 200.0, 300.0, 500.0, 750.0, 1000.0],
                y_values: vec![0.5, 0.52, 0.55, 0.61, 0.68, 0.73, 0.78],
                unit: "freq".into(),
            },
        ),
        (
            "rna_mi_covariance",
            DataBinding::Heatmap {
                id: "bl_rna_mi_cov".into(),
                label: "RNA Mutual Information".into(),
                x_labels: vec![
                    "pos1".into(),
                    "pos2".into(),
                    "pos3".into(),
                    "pos4".into(),
                    "pos5".into(),
                ],
                y_labels: vec![
                    "pos1".into(),
                    "pos2".into(),
                    "pos3".into(),
                    "pos4".into(),
                    "pos5".into(),
                ],
                values: vec![
                    1.0, 0.8, 0.2, 0.1, 0.05, 0.8, 1.0, 0.3, 0.15, 0.1, 0.2, 0.3, 1.0, 0.7, 0.4,
                    0.1, 0.15, 0.7, 1.0, 0.6, 0.05, 0.1, 0.4, 0.6, 1.0,
                ],
                unit: "bits".into(),
            },
        ),
        (
            "rna_mi_entropy",
            DataBinding::Spectrum {
                id: "bl_rna_mi_entropy".into(),
                label: "Positional Entropy".into(),
                frequencies: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
                amplitudes: vec![0.3, 0.7, 0.5, 0.9, 0.2, 0.6, 0.4, 0.8],
                unit: "bits".into(),
            },
        ),
    ]
}
