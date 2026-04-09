// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(missing_docs)]
//! Benchmarks for the Grammar of Graphics compiler pipeline.
//!
//! Measures grammar expression compilation and scene graph construction throughput.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use petal_tongue_scene::grammar::{ScaleBinding, VariableBinding, VariableRole};
use petal_tongue_scene::{CoordinateSystem, GeometryType, GrammarCompiler, GrammarExpr, ScaleType};

#[expect(
    clippy::cast_precision_loss,
    reason = "bench loop indices fit in f64 exactly (n ≤ 10k ≪ 2^53 mantissa)"
)]
fn sample_data(n: usize) -> Vec<serde_json::Value> {
    (0..n)
        .map(|i| {
            serde_json::json!({
                "x": i as f64,
                "y": (i as f64).sin(),
            })
        })
        .collect()
}

fn sample_grammar() -> GrammarExpr {
    GrammarExpr {
        data_source: "bench".to_string(),
        variables: vec![
            VariableBinding {
                name: "x".to_string(),
                field: "x".to_string(),
                role: VariableRole::X,
            },
            VariableBinding {
                name: "y".to_string(),
                field: "y".to_string(),
                role: VariableRole::Y,
            },
        ],
        geometry: GeometryType::Point,
        scales: vec![
            ScaleBinding {
                variable: "x".to_string(),
                scale_type: ScaleType::Linear,
            },
            ScaleBinding {
                variable: "y".to_string(),
                scale_type: ScaleType::Linear,
            },
        ],
        coordinate: CoordinateSystem::Cartesian,
        facets: None,
        aesthetics: Vec::new(),
        title: None,
        domain: None,
    }
}

fn bench_compile_100(c: &mut Criterion) {
    let expr = sample_grammar();
    let data = sample_data(100);
    let compiler = GrammarCompiler::new();
    c.bench_function("grammar_compile_100_points", |b| {
        b.iter(|| compiler.compile(black_box(&expr), black_box(&data)));
    });
}

fn bench_compile_1k(c: &mut Criterion) {
    let expr = sample_grammar();
    let data = sample_data(1_000);
    let compiler = GrammarCompiler::new();
    c.bench_function("grammar_compile_1k_points", |b| {
        b.iter(|| compiler.compile(black_box(&expr), black_box(&data)));
    });
}

fn bench_compile_10k(c: &mut Criterion) {
    let expr = sample_grammar();
    let data = sample_data(10_000);
    let compiler = GrammarCompiler::new();
    c.bench_function("grammar_compile_10k_points", |b| {
        b.iter(|| compiler.compile(black_box(&expr), black_box(&data)));
    });
}

criterion_group!(
    benches,
    bench_compile_100,
    bench_compile_1k,
    bench_compile_10k
);
criterion_main!(benches);
