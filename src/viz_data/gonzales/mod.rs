// SPDX-License-Identifier: AGPL-3.0-or-later
//! Gonzales Interactive Explorer chart scenes.
//!
//! Four pharmacological visualization scenes:
//! - **IC50**: Sigmoidal dose-response (4-parameter logistic Hill equation)
//! - **PK decay**: Two-compartment pharmacokinetic elimination
//! - **Tissue lattice**: Spatial cell-state grid (viability heat map)
//! - **Hormesis**: Biphasic dose-response (low-dose stimulation, high-dose inhibition)
//!
//! Each scene uses the petal-tongue-scene math layer (`Axes`, `FunctionPlot`)
//! and renders to a `SceneGraph` compatible with all modalities.

mod hormesis;
mod ic50;
mod pk_decay;
mod tissue_lattice;

pub use hormesis::build_hormesis_scene;
pub use ic50::build_ic50_scene;
pub use pk_decay::build_pk_decay_scene;
pub use tissue_lattice::build_tissue_lattice_scene;

use petal_tongue_scene::animation::{Animation, AnimationTarget, Easing, Sequence};

// ── Animations ──────────────────────────────────────────────────────────────

/// Build a dose-sweep animation for IC50 (traces the curve left to right).
pub fn build_ic50_sweep_animation() -> Sequence {
    Sequence::Sequential(vec![Animation {
        target: AnimationTarget::Opacity {
            node_id: "ic50-curve".to_owned(),
            from: 0.0,
            to: 1.0,
        },
        duration_secs: 2.0,
        delay_secs: 0.5,
        easing: Easing::EaseInOut,
    }])
}

/// Build a time-lapse animation for PK decay (reveals concentration drop).
pub fn build_pk_decay_animation() -> Sequence {
    Sequence::Sequential(vec![
        Animation {
            target: AnimationTarget::Opacity {
                node_id: "pk-total".to_owned(),
                from: 0.0,
                to: 1.0,
            },
            duration_secs: 1.5,
            delay_secs: 0.0,
            easing: Easing::EaseOut,
        },
        Animation {
            target: AnimationTarget::Opacity {
                node_id: "pk-alpha".to_owned(),
                from: 0.0,
                to: 1.0,
            },
            duration_secs: 1.0,
            delay_secs: 0.3,
            easing: Easing::EaseIn,
        },
        Animation {
            target: AnimationTarget::Opacity {
                node_id: "pk-beta".to_owned(),
                from: 0.0,
                to: 1.0,
            },
            duration_secs: 1.0,
            delay_secs: 0.3,
            easing: Easing::EaseIn,
        },
    ])
}

/// Build a drug-diffusion animation for the tissue lattice.
pub fn build_tissue_diffusion_animation() -> Sequence {
    Sequence::Sequential(vec![Animation {
        target: AnimationTarget::Opacity {
            node_id: "tissue-grid".to_owned(),
            from: 0.0,
            to: 1.0,
        },
        duration_secs: 3.0,
        delay_secs: 0.3,
        easing: Easing::EaseInOut,
    }])
}

/// Build a dose-sweep animation for hormesis.
pub fn build_hormesis_sweep_animation() -> Sequence {
    Sequence::Sequential(vec![Animation {
        target: AnimationTarget::Opacity {
            node_id: "hormesis-curve".to_owned(),
            from: 0.0,
            to: 1.0,
        },
        duration_secs: 2.5,
        delay_secs: 0.5,
        easing: Easing::EaseInOut,
    }])
}

#[cfg(test)]
#[cfg_attr(test, allow(clippy::unwrap_used))]
mod tests {
    use super::*;
    use tissue_lattice::viability_color;

    #[test]
    fn ic50_scene_has_nodes_and_primitives() {
        let scene = build_ic50_scene();
        assert!(scene.node_count() > 3);
        assert!(scene.total_primitives() > 10);
    }

    #[test]
    fn ic50_hill_equation_bounds() {
        let params = ic50::HillParams {
            top: 100.0,
            bottom: 5.0,
            ic50: 1e-7,
            hill_coeff: 1.2,
        };
        let high = params.response(-12.0);
        assert!(high > 95.0, "low conc should give high response: {high}");
        let low = params.response(-3.0);
        assert!(low < 15.0, "high conc should give low response: {low}");
        let mid = params.response(params.ic50.log10());
        let expected_mid = f64::midpoint(params.top, params.bottom);
        assert!(
            (mid - expected_mid).abs() < 5.0,
            "at IC50 should be near midpoint: {mid} vs {expected_mid}"
        );
    }

    #[test]
    fn pk_decay_scene_has_multiple_curves() {
        let scene = build_pk_decay_scene();
        assert!(scene.node_count() > 5);
        assert!(scene.total_primitives() > 10);
    }

    #[test]
    fn pk_two_compartment_decay() {
        let params = pk_decay::PkParams {
            a_coeff: 80.0,
            alpha: 1.5,
            b_coeff: 20.0,
            beta: 0.15,
        };
        let c0 = params.concentration(0.0);
        assert!((c0 - 100.0).abs() < 1e-10, "C(0) = A + B = 100: {c0}");
        let c1 = params.concentration(1.0);
        let c5 = params.concentration(5.0);
        let c24 = params.concentration(24.0);
        assert!(c0 > c1 && c1 > c5 && c5 > c24);
        assert!(c24 < 5.0, "C(24h) should be near zero: {c24}");
    }

    #[test]
    fn tissue_lattice_scene_has_grid() {
        let scene = build_tissue_lattice_scene();
        let grid_node = scene.get("tissue-grid").expect("grid node exists");
        assert_eq!(grid_node.primitives.len(), 144);
    }

    #[test]
    fn tissue_viability_color_range() {
        let dead = viability_color(0.0);
        let alive = viability_color(1.0);
        assert!(dead.r > 0.5, "dead cells should be reddish");
        assert!(alive.g > alive.r, "alive cells should be greenish");
    }

    #[test]
    fn hormesis_scene_has_baseline_and_curve() {
        let scene = build_hormesis_scene();
        assert!(scene.node_count() > 4);
        assert!(scene.total_primitives() > 10);
    }

    #[test]
    fn hormesis_biphasic_shape() {
        let params = hormesis::HormesisParams {
            baseline: 100.0,
            stimulation_peak: 25.0,
            peak_dose: 1e-8,
            inhibition_ic50: 1e-5,
            hill_n: 1.5,
        };
        let low = params.response(-11.0);
        assert!(
            (low - 100.0).abs() < 10.0,
            "very low dose near baseline: {low}"
        );
        let peak = params.response(params.peak_dose.log10());
        assert!(peak > 100.0, "peak should exceed baseline: {peak}");
        let high = params.response(-3.0);
        assert!(high < 100.0, "high dose should be inhibited: {high}");
    }

    #[test]
    fn animations_produce_valid_sequences() {
        let ic50_anim = build_ic50_sweep_animation();
        assert!(ic50_anim.total_duration() > 0.0);
        let pk_anim = build_pk_decay_animation();
        assert!(pk_anim.total_duration() > 2.0);
        let tissue_anim = build_tissue_diffusion_animation();
        assert!(tissue_anim.total_duration() > 0.0);
        let hormesis_anim = build_hormesis_sweep_animation();
        assert!(hormesis_anim.total_duration() > 0.0);
    }
}
