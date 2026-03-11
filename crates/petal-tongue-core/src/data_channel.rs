// SPDX-License-Identifier: AGPL-3.0-only
//! Data binding and threshold range types for universal visualization.
//!
//! Schema for binding data sources to visualizations across all spring domains:
//! timeseries, distribution, bar, gauge, heatmap, scatter, scatter3d, field map, spectrum.
//! Optional threshold ranges for normal/warning/critical state coloring.

use serde::{Deserialize, Serialize};

/// Data binding for universal visualization (binds data source to chart type)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "channel_type")]
pub enum DataBinding {
    /// Time-series plot of a metric over time (e.g., glucose, blood pressure).
    #[serde(rename = "timeseries")]
    TimeSeries {
        /// Unique identifier for this channel within the visualization.
        id: String,
        /// Human-readable display name for the metric.
        label: String,
        /// Axis label for the independent variable (typically time).
        x_label: String,
        /// Axis label for the dependent variable (the measured metric).
        y_label: String,
        /// Unit of measurement for the y-axis values.
        unit: String,
        /// Independent variable values (e.g., timestamps or sample indices).
        x_values: Vec<f64>,
        /// Measured values corresponding to each x-value.
        y_values: Vec<f64>,
    },
    /// Distribution plot comparing a value against a reference population.
    #[serde(rename = "distribution")]
    Distribution {
        /// Unique identifier for this channel within the visualization.
        id: String,
        /// Human-readable display name for the metric.
        label: String,
        /// Unit of measurement for the values.
        unit: String,
        /// Reference population values used to compute the distribution.
        values: Vec<f64>,
        /// Population mean (reference center).
        mean: f64,
        /// Population standard deviation (spread of reference).
        std: f64,
        /// The value to compare against the distribution (e.g., current measurement).
        #[serde(alias = "patient_value")]
        comparison_value: f64,
    },
    /// Bar chart for categorical comparisons (e.g., lab panels, multi-metric summaries).
    #[serde(rename = "bar")]
    Bar {
        /// Unique identifier for this channel within the visualization.
        id: String,
        /// Human-readable display name for the chart.
        label: String,
        /// Category names for each bar (e.g., lab test names or time periods).
        categories: Vec<String>,
        /// Numeric value for each category.
        values: Vec<f64>,
        /// Unit of measurement for the values.
        unit: String,
    },
    /// Gauge or meter display for a single value within reference bounds.
    #[serde(rename = "gauge")]
    Gauge {
        /// Unique identifier for this channel within the visualization.
        id: String,
        /// Human-readable display name for the metric.
        label: String,
        /// Current measured value to display on the gauge.
        value: f64,
        /// Minimum scale value for the gauge axis.
        min: f64,
        /// Maximum scale value for the gauge axis.
        max: f64,
        /// Unit of measurement for the value.
        unit: String,
        /// Reference range [low, high] considered normal.
        normal_range: [f64; 2],
        /// Range [low, high] that triggers a warning (outside normal but not critical).
        warning_range: [f64; 2],
    },
    /// 2D matrix visualization (e.g., plasma density, attention weights, correlation).
    #[serde(rename = "heatmap")]
    Heatmap {
        /// Unique identifier for this channel within the visualization.
        id: String,
        /// Human-readable display name.
        label: String,
        /// Row labels (y-axis categories or indices).
        x_labels: Vec<String>,
        /// Column labels (x-axis categories or indices).
        y_labels: Vec<String>,
        /// Flattened row-major values: `values[row * x_labels.len() + col]`.
        values: Vec<f64>,
        /// Unit of measurement for cell values.
        unit: String,
    },
    /// 3D scatter plot (e.g., PCoA ordination, phase space, latent embeddings).
    #[serde(rename = "scatter3d")]
    Scatter3D {
        /// Unique identifier for this channel within the visualization.
        id: String,
        /// Human-readable display name.
        label: String,
        /// X coordinates for each point.
        x: Vec<f64>,
        /// Y coordinates for each point.
        y: Vec<f64>,
        /// Z coordinates for each point.
        z: Vec<f64>,
        /// Optional per-point labels (empty vec if unlabeled).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        point_labels: Vec<String>,
        /// X-axis label (e.g., "PC1").
        #[serde(default)]
        x_label: String,
        /// Y-axis label (e.g., "PC2").
        #[serde(default)]
        y_label: String,
        /// Z-axis label (e.g., "PC3").
        #[serde(default)]
        z_label: String,
        /// Unit of measurement for the axes.
        unit: String,
    },
    /// 2D scatter plot (e.g., PCoA ordination, UMAP embedding, KMD plots).
    #[serde(rename = "scatter")]
    Scatter {
        /// Unique identifier for this channel within the visualization.
        id: String,
        /// Human-readable display name.
        label: String,
        /// X coordinates for each point.
        x: Vec<f64>,
        /// Y coordinates for each point.
        y: Vec<f64>,
        /// Optional per-point labels (empty vec if unlabeled).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        point_labels: Vec<String>,
        /// X-axis label (e.g., "PC1 (32.1%)").
        #[serde(default)]
        x_label: String,
        /// Y-axis label (e.g., "PC2 (18.7%)").
        #[serde(default)]
        y_label: String,
        /// Unit of measurement for the axes.
        unit: String,
    },
    /// Spatial field over a regular grid (e.g., ET0 maps, sensor fields, PDE solutions).
    #[serde(rename = "fieldmap")]
    FieldMap {
        /// Unique identifier for this channel within the visualization.
        id: String,
        /// Human-readable display name.
        label: String,
        /// Grid coordinates along the x-axis.
        grid_x: Vec<f64>,
        /// Grid coordinates along the y-axis.
        grid_y: Vec<f64>,
        /// Flattened row-major field values: `values[row * grid_x.len() + col]`.
        values: Vec<f64>,
        /// Unit of measurement for the field values.
        unit: String,
    },
    /// Frequency-domain spectrum (e.g., FFT, HRV power spectrum, noise analysis).
    #[serde(rename = "spectrum")]
    Spectrum {
        /// Unique identifier for this channel within the visualization.
        id: String,
        /// Human-readable display name.
        label: String,
        /// Frequency values (Hz or domain-appropriate unit).
        frequencies: Vec<f64>,
        /// Amplitude or power at each frequency.
        amplitudes: Vec<f64>,
        /// Unit of measurement for amplitudes.
        unit: String,
    },
}

/// Threshold range with status (normal/warning/critical for any metric)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdRange {
    /// Name of the range (e.g., "Normal", "Low", "High").
    pub label: String,
    /// Lower bound of the range in the metric's unit.
    pub min: f64,
    /// Upper bound of the range in the metric's unit.
    pub max: f64,
    /// Interpretation level (e.g., "normal", "warning", "critical").
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::{DataBinding, ThresholdRange};

    /// Representative healthSpring JSON snippet (timeseries, distribution, bar, gauge, ThresholdRange).
    /// Confirms healthSpring schema compatibility with our types.
    const HEALTHSPRING_DATA_CHANNELS_SNIPPET: &str = r#"[
        {
            "channel_type": "timeseries",
            "id": "pk_curve",
            "label": "Oral PK Concentration",
            "x_label": "Time (hr)",
            "y_label": "Concentration (mg/L)",
            "unit": "mg/L",
            "x_values": [0.0, 1.0, 2.0],
            "y_values": [0.0, 0.1, 0.15]
        },
        {
            "channel_type": "distribution",
            "id": "risk_distribution",
            "label": "Population Risk Distribution",
            "unit": "composite risk",
            "values": [0.35, 0.38, 0.36],
            "mean": 0.358,
            "std": 0.01,
            "patient_value": 0.431
        },
        {
            "channel_type": "bar",
            "id": "gut_abundances",
            "label": "Genus Relative Abundance",
            "categories": ["Genus 1", "Genus 2"],
            "values": [0.3, 0.25],
            "unit": "relative"
        },
        {
            "channel_type": "gauge",
            "id": "heart_rate",
            "label": "Heart Rate",
            "value": 72.0,
            "min": 40.0,
            "max": 140.0,
            "unit": "bpm",
            "normal_range": [60.0, 100.0],
            "warning_range": [40.0, 60.0]
        }
    ]"#;

    const HEALTHSPRING_THRESHOLDS_SNIPPET: &str = r#"[
        {"label": "Cmax therapeutic", "min": 0.05, "max": 0.3, "status": "normal"},
        {"label": "Cmax high", "min": 0.3, "max": 0.5, "status": "warning"},
        {"label": "Shannon healthy", "min": 2.5, "max": 4.0, "status": "normal"}
    ]"#;

    #[test]
    fn healthspring_data_channels_round_trip() {
        let bindings: Vec<DataBinding> =
            serde_json::from_str(HEALTHSPRING_DATA_CHANNELS_SNIPPET).expect("deserialize bindings");
        assert_eq!(bindings.len(), 4, "expected 4 DataBinding variants");

        // Spot-check timeseries
        match &bindings[0] {
            DataBinding::TimeSeries {
                id,
                label,
                unit,
                x_values,
                y_values,
                ..
            } => {
                assert_eq!(id, "pk_curve");
                assert_eq!(label, "Oral PK Concentration");
                assert_eq!(unit, "mg/L");
                assert_eq!(x_values.len(), 3);
                assert!((y_values[1] - 0.1).abs() < 1e-9);
            }
            _ => panic!("expected TimeSeries"),
        }

        // Spot-check distribution (patient_value alias)
        match &bindings[1] {
            DataBinding::Distribution {
                id,
                comparison_value,
                mean,
                ..
            } => {
                assert_eq!(id, "risk_distribution");
                assert!((*comparison_value - 0.431).abs() < 1e-9);
                assert!((*mean - 0.358).abs() < 1e-9);
            }
            _ => panic!("expected Distribution"),
        }

        // Spot-check bar
        match &bindings[2] {
            DataBinding::Bar {
                id,
                categories,
                values,
                ..
            } => {
                assert_eq!(id, "gut_abundances");
                assert_eq!(categories.len(), 2);
                assert!((values[0] - 0.3).abs() < 1e-9);
            }
            _ => panic!("expected Bar"),
        }

        // Spot-check gauge
        match &bindings[3] {
            DataBinding::Gauge {
                id,
                value,
                normal_range,
                ..
            } => {
                assert_eq!(id, "heart_rate");
                assert!((*value - 72.0).abs() < 1e-9);
                assert!((normal_range[0] - 60.0).abs() < f64::EPSILON);
                assert!((normal_range[1] - 100.0).abs() < f64::EPSILON);
            }
            _ => panic!("expected Gauge"),
        }
    }

    #[test]
    fn healthspring_threshold_ranges_round_trip() {
        let thresholds: Vec<ThresholdRange> =
            serde_json::from_str(HEALTHSPRING_THRESHOLDS_SNIPPET).expect("deserialize thresholds");
        assert_eq!(thresholds.len(), 3, "expected 3 ThresholdRange items");

        assert_eq!(thresholds[0].label, "Cmax therapeutic");
        assert!((thresholds[0].min - 0.05).abs() < f64::EPSILON);
        assert!((thresholds[0].max - 0.3).abs() < f64::EPSILON);
        assert_eq!(thresholds[0].status, "normal");

        assert_eq!(thresholds[1].status, "warning");
        assert_eq!(thresholds[2].label, "Shannon healthy");
    }

    const WETSPRING_SCATTER_SNIPPET: &str = r#"[
        {
            "channel_type": "scatter",
            "id": "pcoa_ordination",
            "label": "PCoA Ordination",
            "x": [1.2, -0.5, 0.8],
            "y": [0.3, 1.1, -0.7],
            "point_labels": ["Sample A", "Sample B", "Sample C"],
            "x_label": "PC1 (32.1%)",
            "y_label": "PC2 (18.7%)",
            "unit": "eigenvalue"
        },
        {
            "channel_type": "scatter3d",
            "id": "pcoa_3d",
            "label": "PCoA 3D",
            "x": [1.0, 2.0],
            "y": [3.0, 4.0],
            "z": [5.0, 6.0],
            "x_label": "PC1",
            "y_label": "PC2",
            "z_label": "PC3",
            "unit": "eigenvalue"
        }
    ]"#;

    #[test]
    fn wetspring_scatter_round_trip() {
        let bindings: Vec<DataBinding> =
            serde_json::from_str(WETSPRING_SCATTER_SNIPPET).expect("deserialize scatter");
        assert_eq!(bindings.len(), 2);

        match &bindings[0] {
            DataBinding::Scatter {
                id,
                x,
                y,
                point_labels,
                x_label,
                y_label,
                ..
            } => {
                assert_eq!(id, "pcoa_ordination");
                assert_eq!(x.len(), 3);
                assert_eq!(y.len(), 3);
                assert_eq!(point_labels.len(), 3);
                assert_eq!(x_label, "PC1 (32.1%)");
                assert_eq!(y_label, "PC2 (18.7%)");
            }
            _ => panic!("expected Scatter"),
        }

        match &bindings[1] {
            DataBinding::Scatter3D {
                id,
                x_label,
                y_label,
                z_label,
                ..
            } => {
                assert_eq!(id, "pcoa_3d");
                assert_eq!(x_label, "PC1");
                assert_eq!(y_label, "PC2");
                assert_eq!(z_label, "PC3");
            }
            _ => panic!("expected Scatter3D"),
        }
    }

    #[test]
    fn scatter3d_without_labels_deserializes() {
        let json = r#"{
            "channel_type": "scatter3d",
            "id": "legacy",
            "label": "Legacy",
            "x": [1.0],
            "y": [2.0],
            "z": [3.0],
            "unit": "m"
        }"#;
        let binding: DataBinding = serde_json::from_str(json).expect("deserialize");
        match &binding {
            DataBinding::Scatter3D {
                x_label,
                y_label,
                z_label,
                ..
            } => {
                assert!(x_label.is_empty());
                assert!(y_label.is_empty());
                assert!(z_label.is_empty());
            }
            _ => panic!("expected Scatter3D"),
        }
    }

    #[test]
    fn heatmap_round_trip() {
        let json = r#"{
            "channel_type": "heatmap",
            "id": "hm1",
            "label": "Correlation Matrix",
            "x_labels": ["A", "B", "C"],
            "y_labels": ["X", "Y"],
            "values": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            "unit": "corr"
        }"#;
        let binding: DataBinding = serde_json::from_str(json).expect("deserialize");
        match &binding {
            DataBinding::Heatmap {
                id,
                label,
                x_labels,
                y_labels,
                values,
                unit,
            } => {
                assert_eq!(id, "hm1");
                assert_eq!(label, "Correlation Matrix");
                assert_eq!(x_labels.len(), 3);
                assert_eq!(y_labels.len(), 2);
                assert_eq!(values.len(), 6);
                assert_eq!(unit, "corr");
            }
            _ => panic!("expected Heatmap"),
        }
        let serialized = serde_json::to_string(&binding).expect("serialize");
        let restored: DataBinding = serde_json::from_str(&serialized).expect("round-trip");
        assert!(matches!(restored, DataBinding::Heatmap { .. }));
    }

    #[test]
    fn fieldmap_round_trip() {
        let json = r#"{
            "channel_type": "fieldmap",
            "id": "fm1",
            "label": "ET0 Map",
            "grid_x": [0.0, 1.0, 2.0],
            "grid_y": [0.0, 1.0],
            "values": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            "unit": "mm/day"
        }"#;
        let binding: DataBinding = serde_json::from_str(json).expect("deserialize");
        match &binding {
            DataBinding::FieldMap {
                id,
                grid_x,
                grid_y,
                values,
                unit,
                ..
            } => {
                assert_eq!(id, "fm1");
                assert_eq!(grid_x.len(), 3);
                assert_eq!(grid_y.len(), 2);
                assert_eq!(values.len(), 6);
                assert_eq!(unit, "mm/day");
            }
            _ => panic!("expected FieldMap"),
        }
    }

    #[test]
    fn spectrum_round_trip() {
        let json = r#"{
            "channel_type": "spectrum",
            "id": "spec1",
            "label": "Power Spectrum",
            "frequencies": [0.0, 1.0, 2.0, 3.0],
            "amplitudes": [0.1, 0.5, 0.3, 0.05],
            "unit": "dB"
        }"#;
        let binding: DataBinding = serde_json::from_str(json).expect("deserialize");
        match &binding {
            DataBinding::Spectrum {
                id,
                frequencies,
                amplitudes,
                unit,
                ..
            } => {
                assert_eq!(id, "spec1");
                assert_eq!(frequencies.len(), 4);
                assert_eq!(amplitudes.len(), 4);
                assert!((amplitudes[1] - 0.5).abs() < f64::EPSILON);
                assert_eq!(unit, "dB");
            }
            _ => panic!("expected Spectrum"),
        }
    }

    #[test]
    fn threshold_range_serialization_round_trip() {
        let tr = ThresholdRange {
            label: "Critical".to_string(),
            min: 0.0,
            max: 1.0,
            status: "critical".to_string(),
        };
        let json = serde_json::to_string(&tr).expect("serialize");
        let restored: ThresholdRange = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.label, "Critical");
        assert!((restored.min - 0.0).abs() < f64::EPSILON);
        assert!((restored.max - 1.0).abs() < f64::EPSILON);
        assert_eq!(restored.status, "critical");
    }

    #[test]
    fn data_binding_invalid_json_fails() {
        let result = serde_json::from_str::<DataBinding>("{invalid}");
        assert!(result.is_err());
    }

    #[test]
    fn data_binding_unknown_channel_type_fails() {
        let result = serde_json::from_str::<DataBinding>(
            r#"{"channel_type": "unknown", "id": "x", "label": "x"}"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn scatter_without_point_labels_deserializes() {
        let json = r#"{
            "channel_type": "scatter",
            "id": "s1",
            "label": "Scatter",
            "x": [1.0, 2.0],
            "y": [3.0, 4.0],
            "unit": "u"
        }"#;
        let binding: DataBinding = serde_json::from_str(json).expect("deserialize");
        match &binding {
            DataBinding::Scatter {
                point_labels,
                x_label,
                y_label,
                ..
            } => {
                assert!(point_labels.is_empty());
                assert!(x_label.is_empty());
                assert!(y_label.is_empty());
            }
            _ => panic!("expected Scatter"),
        }
    }
}
