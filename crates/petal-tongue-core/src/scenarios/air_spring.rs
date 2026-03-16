// SPDX-License-Identifier: AGPL-3.0-only
//! Scenario builders for airSpring capabilities.
//!
//! airSpring exposes 35 `science.*` capabilities (ET0, soil, crop, drought,
//! biodiversity) via JSON-RPC but has no petalTongue scenario builders.
//! These builders generate representative visualization data for each domain.

use crate::DataBinding;
use crate::scenario_builder::{ScenarioBuilder, ScenarioMetadata, VisualizationScene};

/// ET0 (reference evapotranspiration) time series scenario.
pub struct AirSpringET0Scenario;

impl ScenarioBuilder for AirSpringET0Scenario {
    fn id(&self) -> &'static str {
        "airspring.et0"
    }

    fn name(&self) -> &'static str {
        "ET0 Reference Evapotranspiration"
    }

    fn domain(&self) -> &'static str {
        "agriculture"
    }

    fn available_scenes(&self) -> Vec<String> {
        vec!["daily_et0".to_string(), "monthly_et0".to_string()]
    }

    fn build_scene(&self, scene_name: &str) -> Option<VisualizationScene> {
        match scene_name {
            "daily_et0" => Some(build_daily_et0()),
            "monthly_et0" => Some(build_monthly_et0()),
            _ => None,
        }
    }
}

fn build_daily_et0() -> VisualizationScene {
    let meta = ScenarioMetadata {
        title: "Daily ET0 Reference".to_string(),
        description: "Penman-Monteith ET0 over 30 days".to_string(),
        version: "1.0.0".to_string(),
        domain: "agriculture".to_string(),
    };
    let days: Vec<f64> = (0..30).map(f64::from).collect();
    let et0_values: Vec<f64> = days
        .iter()
        .map(|d| 1.5f64.mul_add((d * std::f64::consts::TAU / 30.0).sin(), 3.0))
        .collect();
    VisualizationScene::new(meta).with_binding(DataBinding::TimeSeries {
        id: "daily_et0".to_string(),
        label: "ET0".to_string(),
        x_label: "Day".to_string(),
        y_label: "ET0 (mm/day)".to_string(),
        unit: "mm/day".to_string(),
        x_values: days,
        y_values: et0_values,
    })
}

fn build_monthly_et0() -> VisualizationScene {
    let meta = ScenarioMetadata {
        title: "Monthly ET0 Summary".to_string(),
        description: "Monthly aggregated ET0 bar chart".to_string(),
        version: "1.0.0".to_string(),
        domain: "agriculture".to_string(),
    };
    VisualizationScene::new(meta).with_binding(DataBinding::Bar {
        id: "monthly_et0".to_string(),
        label: "Monthly ET0".to_string(),
        categories: vec![
            "Jan".into(),
            "Feb".into(),
            "Mar".into(),
            "Apr".into(),
            "May".into(),
            "Jun".into(),
            "Jul".into(),
            "Aug".into(),
            "Sep".into(),
            "Oct".into(),
            "Nov".into(),
            "Dec".into(),
        ],
        values: vec![
            45.0, 55.0, 80.0, 105.0, 135.0, 155.0, 165.0, 150.0, 120.0, 90.0, 60.0, 40.0,
        ],
        unit: "mm/month".to_string(),
    })
}

/// Richards PDE soil moisture field map scenario.
pub struct AirSpringRichardsPDEScenario;

impl ScenarioBuilder for AirSpringRichardsPDEScenario {
    fn id(&self) -> &'static str {
        "airspring.richards_pde"
    }

    fn name(&self) -> &'static str {
        "Richards PDE Soil Moisture"
    }

    fn domain(&self) -> &'static str {
        "agriculture"
    }

    fn available_scenes(&self) -> Vec<String> {
        vec!["moisture_field".to_string()]
    }

    fn build_scene(&self, scene_name: &str) -> Option<VisualizationScene> {
        match scene_name {
            "moisture_field" => Some(build_richards_field()),
            _ => None,
        }
    }
}

fn build_richards_field() -> VisualizationScene {
    let meta = ScenarioMetadata {
        title: "Richards PDE Soil Moisture".to_string(),
        description: "2D soil moisture field from Richards equation solution".to_string(),
        version: "1.0.0".to_string(),
        domain: "agriculture".to_string(),
    };
    let nx = 10;
    let ny = 8;
    let grid_x: Vec<f64> = (0..nx).map(|i| f64::from(i) * 0.1).collect();
    let grid_y: Vec<f64> = (0..ny).map(|i| f64::from(i) * 0.05).collect();
    let values: Vec<f64> = (0..ny)
        .flat_map(|row| {
            (0..nx).map(move |col| {
                let depth = f64::from(row) / f64::from(ny);
                let lateral = f64::from(col) / f64::from(nx);
                0.05f64.mul_add(
                    (lateral * std::f64::consts::PI).sin(),
                    0.15f64.mul_add(-depth, 0.35),
                )
            })
        })
        .collect();
    VisualizationScene::new(meta).with_binding(DataBinding::FieldMap {
        id: "richards_moisture".to_string(),
        label: "Soil Moisture (θ)".to_string(),
        grid_x,
        grid_y,
        values,
        unit: "m³/m³".to_string(),
    })
}

/// Crop coefficient gauge scenario.
pub struct AirSpringCropCoefficientScenario;

impl ScenarioBuilder for AirSpringCropCoefficientScenario {
    fn id(&self) -> &'static str {
        "airspring.crop_coefficient"
    }

    fn name(&self) -> &'static str {
        "Crop Coefficient (Kc)"
    }

    fn domain(&self) -> &'static str {
        "agriculture"
    }

    fn available_scenes(&self) -> Vec<String> {
        vec!["kc_gauge".to_string(), "kc_stages".to_string()]
    }

    fn build_scene(&self, scene_name: &str) -> Option<VisualizationScene> {
        match scene_name {
            "kc_gauge" => Some(build_kc_gauge()),
            "kc_stages" => Some(build_kc_stages()),
            _ => None,
        }
    }
}

fn build_kc_gauge() -> VisualizationScene {
    let meta = ScenarioMetadata {
        title: "Crop Coefficient Gauge".to_string(),
        description: "Current Kc value with growth stage ranges".to_string(),
        version: "1.0.0".to_string(),
        domain: "agriculture".to_string(),
    };
    VisualizationScene::new(meta).with_binding(DataBinding::Gauge {
        id: "kc_current".to_string(),
        label: "Kc (Maize)".to_string(),
        value: 0.85,
        min: 0.0,
        max: 1.5,
        unit: "dimensionless".to_string(),
        normal_range: [0.3, 1.2],
        warning_range: [0.0, 1.5],
    })
}

fn build_kc_stages() -> VisualizationScene {
    let meta = ScenarioMetadata {
        title: "Crop Coefficient by Growth Stage".to_string(),
        description: "Kc progression through FAO-56 growth stages".to_string(),
        version: "1.0.0".to_string(),
        domain: "agriculture".to_string(),
    };
    VisualizationScene::new(meta).with_binding(DataBinding::Bar {
        id: "kc_stages".to_string(),
        label: "Kc by Stage".to_string(),
        categories: vec![
            "Initial".into(),
            "Development".into(),
            "Mid-season".into(),
            "Late".into(),
        ],
        values: vec![0.3, 0.7, 1.15, 0.35],
        unit: "Kc".to_string(),
    })
}

/// Drought index bar chart scenario.
pub struct AirSpringDroughtIndexScenario;

impl ScenarioBuilder for AirSpringDroughtIndexScenario {
    fn id(&self) -> &'static str {
        "airspring.drought_index"
    }

    fn name(&self) -> &'static str {
        "Drought Index (SPI/SPEI)"
    }

    fn domain(&self) -> &'static str {
        "agriculture"
    }

    fn available_scenes(&self) -> Vec<String> {
        vec!["spi_timeseries".to_string()]
    }

    fn build_scene(&self, scene_name: &str) -> Option<VisualizationScene> {
        match scene_name {
            "spi_timeseries" => Some(build_spi_timeseries()),
            _ => None,
        }
    }
}

fn build_spi_timeseries() -> VisualizationScene {
    let meta = ScenarioMetadata {
        title: "Standardized Precipitation Index".to_string(),
        description: "12-month SPI time series".to_string(),
        version: "1.0.0".to_string(),
        domain: "agriculture".to_string(),
    };
    let months: Vec<f64> = (0..24).map(f64::from).collect();
    let spi: Vec<f64> = months
        .iter()
        .map(|m| 0.5f64.mul_add((m * std::f64::consts::TAU / 12.0).sin(), -0.3))
        .collect();
    VisualizationScene::new(meta).with_binding(DataBinding::TimeSeries {
        id: "spi_12".to_string(),
        label: "SPI-12".to_string(),
        x_label: "Month".to_string(),
        y_label: "SPI".to_string(),
        unit: "σ".to_string(),
        x_values: months,
        y_values: spi,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn et0_scenario_metadata() {
        let builder = AirSpringET0Scenario;
        assert_eq!(builder.id(), "airspring.et0");
        assert_eq!(builder.domain(), "agriculture");
        assert_eq!(builder.available_scenes().len(), 2);
    }

    #[test]
    fn et0_daily_scene_has_data() {
        let builder = AirSpringET0Scenario;
        let scene = builder.build_scene("daily_et0").unwrap();
        assert_eq!(scene.bindings.len(), 1);
        match &scene.bindings[0] {
            DataBinding::TimeSeries { x_values, .. } => assert_eq!(x_values.len(), 30),
            _ => panic!("expected TimeSeries"),
        }
    }

    #[test]
    fn et0_monthly_scene_has_12_months() {
        let builder = AirSpringET0Scenario;
        let scene = builder.build_scene("monthly_et0").unwrap();
        match &scene.bindings[0] {
            DataBinding::Bar { categories, .. } => assert_eq!(categories.len(), 12),
            _ => panic!("expected Bar"),
        }
    }

    #[test]
    fn richards_pde_produces_field_map() {
        let builder = AirSpringRichardsPDEScenario;
        let scene = builder.build_scene("moisture_field").unwrap();
        match &scene.bindings[0] {
            DataBinding::FieldMap {
                grid_x,
                grid_y,
                values,
                ..
            } => {
                assert_eq!(grid_x.len(), 10);
                assert_eq!(grid_y.len(), 8);
                assert_eq!(values.len(), 80);
            }
            _ => panic!("expected FieldMap"),
        }
    }

    #[test]
    fn crop_coefficient_gauge() {
        let builder = AirSpringCropCoefficientScenario;
        let scene = builder.build_scene("kc_gauge").unwrap();
        match &scene.bindings[0] {
            DataBinding::Gauge { value, .. } => {
                assert!((*value - 0.85).abs() < f64::EPSILON);
            }
            _ => panic!("expected Gauge"),
        }
    }

    #[test]
    fn crop_coefficient_stages() {
        let builder = AirSpringCropCoefficientScenario;
        let scene = builder.build_scene("kc_stages").unwrap();
        match &scene.bindings[0] {
            DataBinding::Bar { categories, .. } => assert_eq!(categories.len(), 4),
            _ => panic!("expected Bar"),
        }
    }

    #[test]
    fn drought_index_timeseries() {
        let builder = AirSpringDroughtIndexScenario;
        let scene = builder.build_scene("spi_timeseries").unwrap();
        match &scene.bindings[0] {
            DataBinding::TimeSeries { x_values, .. } => assert_eq!(x_values.len(), 24),
            _ => panic!("expected TimeSeries"),
        }
    }

    #[test]
    fn build_all_returns_all_scenes() {
        let builder = AirSpringET0Scenario;
        let scenes = builder.build_all();
        assert_eq!(scenes.len(), 2);
    }

    #[test]
    fn et0_daily_formula() {
        let builder = AirSpringET0Scenario;
        let scene = builder.build_scene("daily_et0").unwrap();
        match &scene.bindings[0] {
            DataBinding::TimeSeries {
                x_values, y_values, ..
            } => {
                let d = x_values[0];
                let expected = 1.5f64.mul_add((d * std::f64::consts::TAU / 30.0).sin(), 3.0);
                assert!((y_values[0] - expected).abs() < 1e-10);
            }
            _ => panic!("expected TimeSeries"),
        }
    }

    #[test]
    #[expect(
        clippy::cast_precision_loss,
        reason = "small test indices well within f64 precision"
    )]
    fn richards_field_formula() {
        let builder = AirSpringRichardsPDEScenario;
        let scene = builder.build_scene("moisture_field").unwrap();
        match &scene.bindings[0] {
            DataBinding::FieldMap {
                grid_x,
                grid_y,
                values,
                ..
            } => {
                let row = 0_usize;
                let col = 0_usize;
                let depth = row as f64 / grid_y.len() as f64;
                let lateral = col as f64 / grid_x.len() as f64;
                let expected = 0.05f64.mul_add(
                    (lateral * std::f64::consts::PI).sin(),
                    0.15f64.mul_add(-depth, 0.35),
                );
                let idx = row * grid_x.len() + col;
                assert!((values[idx] - expected).abs() < 1e-10);
            }
            _ => panic!("expected FieldMap"),
        }
    }

    #[test]
    fn spi_timeseries_formula() {
        let builder = AirSpringDroughtIndexScenario;
        let scene = builder.build_scene("spi_timeseries").unwrap();
        match &scene.bindings[0] {
            DataBinding::TimeSeries {
                x_values, y_values, ..
            } => {
                let m = x_values[0];
                let expected = 0.5f64.mul_add((m * std::f64::consts::TAU / 12.0).sin(), -0.3);
                assert!((y_values[0] - expected).abs() < 1e-10);
            }
            _ => panic!("expected TimeSeries"),
        }
    }

    #[test]
    fn unknown_scene_returns_none() {
        let builder = AirSpringET0Scenario;
        assert!(builder.build_scene("nonexistent").is_none());
    }
}
