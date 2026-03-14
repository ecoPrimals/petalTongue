// SPDX-License-Identifier: AGPL-3.0-only
//! Scenario builders for groundSpring capabilities.
//!
//! groundSpring has 395 validation checks and 102 barraCuda delegations for
//! physics (seismic, spectral, Anderson localization, sensor calibration) but
//! no petalTongue scenario builders. These fill the visualization gap.

use crate::scenario_builder::{ScenarioBuilder, ScenarioMetadata, VisualizationScene};
use crate::{DataBinding, ThresholdRange};

/// Seismic wave field map scenario.
pub struct GroundSpringSeismicScenario;

impl ScenarioBuilder for GroundSpringSeismicScenario {
    fn id(&self) -> &'static str {
        "groundspring.seismic"
    }

    fn name(&self) -> &'static str {
        "Seismic Wave Propagation"
    }

    fn domain(&self) -> &'static str {
        "measurement"
    }

    fn available_scenes(&self) -> Vec<String> {
        vec![
            "wave_field".to_string(),
            "arrival_times".to_string(),
        ]
    }

    fn build_scene(&self, scene_name: &str) -> Option<VisualizationScene> {
        match scene_name {
            "wave_field" => Some(build_seismic_field()),
            "arrival_times" => Some(build_arrival_times()),
            _ => None,
        }
    }
}

fn build_seismic_field() -> VisualizationScene {
    let meta = ScenarioMetadata {
        title: "Seismic Wave Field".to_string(),
        description: "2D P-wave amplitude field snapshot".to_string(),
        version: "1.0.0".to_string(),
        domain: "measurement".to_string(),
    };
    let nx = 12;
    let ny = 10;
    let grid_x: Vec<f64> = (0..nx).map(|i| f64::from(i) * 100.0).collect();
    let grid_y: Vec<f64> = (0..ny).map(|i| f64::from(i) * 50.0).collect();
    let cx = 600.0_f64;
    let cy = 250.0_f64;
    let values: Vec<f64> = (0..ny)
        .flat_map(|row| {
            let y = f64::from(row) * 50.0;
            (0..nx).map(move |col| {
                let x = f64::from(col) * 100.0;
                let r = (x - cx).hypot(y - cy);
                (-(r / 200.0).powi(2)).exp() * (r / 80.0).sin()
            })
        })
        .collect();
    VisualizationScene::new(meta).with_binding(DataBinding::FieldMap {
        id: "seismic_field".to_string(),
        label: "P-Wave Amplitude".to_string(),
        grid_x,
        grid_y,
        values,
        unit: "m/s²".to_string(),
    })
}

fn build_arrival_times() -> VisualizationScene {
    let meta = ScenarioMetadata {
        title: "Seismic Arrival Times".to_string(),
        description: "P-wave arrival times across sensor array".to_string(),
        version: "1.0.0".to_string(),
        domain: "measurement".to_string(),
    };
    let sensors: Vec<f64> = (0..16).map(f64::from).collect();
    let times: Vec<f64> = sensors
        .iter()
        .map(|s| 0.5 + 0.1 * s + 0.02 * (s * 0.5).sin())
        .collect();
    VisualizationScene::new(meta).with_binding(DataBinding::TimeSeries {
        id: "arrival_times".to_string(),
        label: "P-Wave Arrival".to_string(),
        x_label: "Sensor Index".to_string(),
        y_label: "Time (s)".to_string(),
        unit: "s".to_string(),
        x_values: sensors,
        y_values: times,
    })
}

/// Sensor drift time series scenario.
pub struct GroundSpringSensorDriftScenario;

impl ScenarioBuilder for GroundSpringSensorDriftScenario {
    fn id(&self) -> &'static str {
        "groundspring.sensor_drift"
    }

    fn name(&self) -> &'static str {
        "Sensor Calibration Drift"
    }

    fn domain(&self) -> &'static str {
        "measurement"
    }

    fn available_scenes(&self) -> Vec<String> {
        vec!["drift_timeseries".to_string()]
    }

    fn build_scene(&self, scene_name: &str) -> Option<VisualizationScene> {
        match scene_name {
            "drift_timeseries" => Some(build_sensor_drift()),
            _ => None,
        }
    }
}

fn build_sensor_drift() -> VisualizationScene {
    let meta = ScenarioMetadata {
        title: "Sensor Drift Analysis".to_string(),
        description: "Accelerometer calibration drift over 90 days".to_string(),
        version: "1.0.0".to_string(),
        domain: "measurement".to_string(),
    };
    let days: Vec<f64> = (0..90).map(f64::from).collect();
    let drift: Vec<f64> = days
        .iter()
        .map(|d| 0.001 * d + 0.0005 * (d * 0.1).sin())
        .collect();
    VisualizationScene::new(meta)
        .with_binding(DataBinding::TimeSeries {
            id: "drift".to_string(),
            label: "Sensor Drift".to_string(),
            x_label: "Day".to_string(),
            y_label: "Offset (g)".to_string(),
            unit: "g".to_string(),
            x_values: days,
            y_values: drift,
        })
        .with_threshold(ThresholdRange {
            label: "Acceptable drift".to_string(),
            min: -0.05,
            max: 0.05,
            status: "normal".to_string(),
        })
        .with_threshold(ThresholdRange {
            label: "Recalibration needed".to_string(),
            min: 0.05,
            max: 0.1,
            status: "warning".to_string(),
        })
}

/// Spectral reconstruction spectrum scenario.
pub struct GroundSpringSpectralReconstructionScenario;

impl ScenarioBuilder for GroundSpringSpectralReconstructionScenario {
    fn id(&self) -> &'static str {
        "groundspring.spectral_reconstruction"
    }

    fn name(&self) -> &'static str {
        "Spectral Reconstruction"
    }

    fn domain(&self) -> &'static str {
        "measurement"
    }

    fn available_scenes(&self) -> Vec<String> {
        vec!["power_spectrum".to_string()]
    }

    fn build_scene(&self, scene_name: &str) -> Option<VisualizationScene> {
        match scene_name {
            "power_spectrum" => Some(build_power_spectrum()),
            _ => None,
        }
    }
}

fn build_power_spectrum() -> VisualizationScene {
    let meta = ScenarioMetadata {
        title: "Reconstructed Power Spectrum".to_string(),
        description: "Ground vibration spectrum from sparse sensor data".to_string(),
        version: "1.0.0".to_string(),
        domain: "measurement".to_string(),
    };
    let n_bins = 64;
    let frequencies: Vec<f64> = (0..n_bins).map(|i| f64::from(i) * 0.5).collect();
    let amplitudes: Vec<f64> = frequencies
        .iter()
        .map(|f| {
            let peak1 = (-((f - 5.0) / 1.5).powi(2)).exp() * 0.8;
            let peak2 = (-((f - 15.0) / 2.0).powi(2)).exp() * 0.4;
            let noise = 0.02;
            peak1 + peak2 + noise
        })
        .collect();
    VisualizationScene::new(meta).with_binding(DataBinding::Spectrum {
        id: "ground_spectrum".to_string(),
        label: "Ground Vibration".to_string(),
        frequencies,
        amplitudes,
        unit: "dB".to_string(),
    })
}

/// Anderson localization heatmap scenario.
pub struct GroundSpringAndersonLocalizationScenario;

impl ScenarioBuilder for GroundSpringAndersonLocalizationScenario {
    fn id(&self) -> &'static str {
        "groundspring.anderson_localization"
    }

    fn name(&self) -> &'static str {
        "Anderson Localization"
    }

    fn domain(&self) -> &'static str {
        "measurement"
    }

    fn available_scenes(&self) -> Vec<String> {
        vec!["localization_heatmap".to_string()]
    }

    fn build_scene(&self, scene_name: &str) -> Option<VisualizationScene> {
        match scene_name {
            "localization_heatmap" => Some(build_anderson_heatmap()),
            _ => None,
        }
    }
}

fn build_anderson_heatmap() -> VisualizationScene {
    let meta = ScenarioMetadata {
        title: "Anderson Localization".to_string(),
        description: "Wave function amplitude in a disordered lattice".to_string(),
        version: "1.0.0".to_string(),
        domain: "measurement".to_string(),
    };
    let size = 8;
    let x_labels: Vec<String> = (0..size).map(|i| format!("x{i}")).collect();
    let y_labels: Vec<String> = (0..size).map(|i| format!("y{i}")).collect();
    let center = f64::from(size) / 2.0;
    let values: Vec<f64> = (0..size)
        .flat_map(|row| {
            (0..size).map(move |col| {
                let dx = f64::from(col) - center;
                let dy = f64::from(row) - center;
                let r2 = dx * dx + dy * dy;
                (-r2 / 4.0).exp()
            })
        })
        .collect();
    VisualizationScene::new(meta).with_binding(DataBinding::Heatmap {
        id: "anderson_loc".to_string(),
        label: "|ψ|²".to_string(),
        x_labels,
        y_labels,
        values,
        unit: "a.u.".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seismic_scenario_metadata() {
        let builder = GroundSpringSeismicScenario;
        assert_eq!(builder.id(), "groundspring.seismic");
        assert_eq!(builder.domain(), "measurement");
        assert_eq!(builder.available_scenes().len(), 2);
    }

    #[test]
    fn seismic_wave_field_is_fieldmap() {
        let builder = GroundSpringSeismicScenario;
        let scene = builder.build_scene("wave_field").unwrap();
        match &scene.bindings[0] {
            DataBinding::FieldMap {
                grid_x,
                grid_y,
                values,
                ..
            } => {
                assert_eq!(grid_x.len(), 12);
                assert_eq!(grid_y.len(), 10);
                assert_eq!(values.len(), 120);
            }
            _ => panic!("expected FieldMap"),
        }
    }

    #[test]
    fn seismic_arrival_times_is_timeseries() {
        let builder = GroundSpringSeismicScenario;
        let scene = builder.build_scene("arrival_times").unwrap();
        match &scene.bindings[0] {
            DataBinding::TimeSeries { x_values, .. } => assert_eq!(x_values.len(), 16),
            _ => panic!("expected TimeSeries"),
        }
    }

    #[test]
    fn sensor_drift_has_thresholds() {
        let builder = GroundSpringSensorDriftScenario;
        let scene = builder.build_scene("drift_timeseries").unwrap();
        assert_eq!(scene.thresholds.len(), 2);
        assert_eq!(scene.thresholds[0].status, "normal");
        assert_eq!(scene.thresholds[1].status, "warning");
    }

    #[test]
    fn sensor_drift_90_days() {
        let builder = GroundSpringSensorDriftScenario;
        let scene = builder.build_scene("drift_timeseries").unwrap();
        match &scene.bindings[0] {
            DataBinding::TimeSeries { x_values, .. } => assert_eq!(x_values.len(), 90),
            _ => panic!("expected TimeSeries"),
        }
    }

    #[test]
    fn spectral_reconstruction_64_bins() {
        let builder = GroundSpringSpectralReconstructionScenario;
        let scene = builder.build_scene("power_spectrum").unwrap();
        match &scene.bindings[0] {
            DataBinding::Spectrum {
                frequencies,
                amplitudes,
                ..
            } => {
                assert_eq!(frequencies.len(), 64);
                assert_eq!(amplitudes.len(), 64);
            }
            _ => panic!("expected Spectrum"),
        }
    }

    #[test]
    fn anderson_localization_is_heatmap() {
        let builder = GroundSpringAndersonLocalizationScenario;
        let scene = builder.build_scene("localization_heatmap").unwrap();
        match &scene.bindings[0] {
            DataBinding::Heatmap {
                x_labels,
                y_labels,
                values,
                ..
            } => {
                assert_eq!(x_labels.len(), 8);
                assert_eq!(y_labels.len(), 8);
                assert_eq!(values.len(), 64);
            }
            _ => panic!("expected Heatmap"),
        }
    }

    #[test]
    fn anderson_localization_values_peak_at_center() {
        let builder = GroundSpringAndersonLocalizationScenario;
        let scene = builder.build_scene("localization_heatmap").unwrap();
        match &scene.bindings[0] {
            DataBinding::Heatmap { values, .. } => {
                let corner = values[0];
                let center_ish = values[4 * 8 + 4];
                assert!(center_ish > corner, "center should have higher amplitude");
            }
            _ => panic!("expected Heatmap"),
        }
    }

    #[test]
    fn build_all_seismic() {
        let builder = GroundSpringSeismicScenario;
        let scenes = builder.build_all();
        assert_eq!(scenes.len(), 2);
    }

    #[test]
    fn unknown_scene_returns_none() {
        let builder = GroundSpringSeismicScenario;
        assert!(builder.build_scene("nonexistent").is_none());
    }
}
