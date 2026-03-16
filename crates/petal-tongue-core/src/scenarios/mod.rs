// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scenario builders for spring domains lacking native petalTongue integration.
//!
//! Springs that already push data via `visualization.render` (neuralSpring,
//! healthSpring, wetSpring) don't need builders here — they provide their own.
//! These builders fill the gap for springs whose science capabilities exist
//! but lack visualization scenario definitions.

pub mod air_spring;
pub mod ground_spring;

pub use air_spring::{
    AirSpringCropCoefficientScenario, AirSpringDroughtIndexScenario, AirSpringET0Scenario,
    AirSpringRichardsPDEScenario,
};
pub use ground_spring::{
    GroundSpringAndersonLocalizationScenario, GroundSpringSeismicScenario,
    GroundSpringSensorDriftScenario, GroundSpringSpectralReconstructionScenario,
};
