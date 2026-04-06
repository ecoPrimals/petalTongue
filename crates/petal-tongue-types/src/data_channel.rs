// SPDX-License-Identifier: AGPL-3.0-or-later
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
    /// 3D scatter plot (e.g., `PCoA` ordination, phase space, latent embeddings).
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
    /// 2D scatter plot (e.g., `PCoA` ordination, UMAP embedding, KMD plots).
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
    /// 2D game scene with tilemap, sprites, and entities.
    ///
    /// Used by ludoSpring and other game-producing primals to push
    /// game state for visualization rendering.
    #[serde(rename = "game_scene")]
    GameScene {
        /// Unique identifier for this channel within the visualization.
        id: String,
        /// Human-readable display name.
        label: String,
        /// The game scene data (tilemap, sprites, entities, camera).
        scene: serde_json::Value,
    },
    /// Soundscape definition: layered audio for ambient/game audio.
    ///
    /// Used by ludoSpring (game audio), wetSpring (ecology ambience),
    /// and other primals that push audio scene data.
    #[serde(rename = "soundscape")]
    Soundscape {
        /// Unique identifier for this channel within the visualization.
        id: String,
        /// Human-readable display name.
        label: String,
        /// The soundscape definition (layers, durations, waveforms).
        definition: serde_json::Value,
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

    #[test]
    fn timeseries_round_trip() {
        let json = r#"{
            "channel_type": "timeseries",
            "id": "pk_curve",
            "label": "Oral PK Concentration",
            "x_label": "Time (hr)",
            "y_label": "Concentration (mg/L)",
            "unit": "mg/L",
            "x_values": [0.0, 1.0, 2.0],
            "y_values": [0.0, 0.1, 0.15]
        }"#;
        let binding: DataBinding = serde_json::from_str(json).expect("deserialize");
        assert!(matches!(binding, DataBinding::TimeSeries { .. }));
        let serialized = serde_json::to_string(&binding).expect("serialize");
        let _restored: DataBinding = serde_json::from_str(&serialized).expect("round-trip");
    }

    #[test]
    fn threshold_range_round_trip() {
        let tr = ThresholdRange {
            label: "Critical".to_string(),
            min: 0.0,
            max: 1.0,
            status: "critical".to_string(),
        };
        let json = serde_json::to_string(&tr).expect("serialize");
        let restored: ThresholdRange = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.label, "Critical");
        assert_eq!(restored.status, "critical");
    }
}
