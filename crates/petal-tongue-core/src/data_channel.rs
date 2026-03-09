// SPDX-License-Identifier: AGPL-3.0-only
//! Data channel and clinical range types from healthSpring
//!
//! Schema for visualization of clinical and timeseries data.

use serde::{Deserialize, Serialize};

/// Data channel for health/clinical visualization (healthSpring schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "channel_type")]
pub enum DataChannel {
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
    /// Distribution plot comparing a patient's value against a reference population.
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
        /// The patient's measured value to compare against the distribution.
        patient_value: f64,
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
        /// Reference range [low, high] considered clinically normal.
        normal_range: [f64; 2],
        /// Range [low, high] that triggers a warning (outside normal but not critical).
        warning_range: [f64; 2],
    },
}

/// Clinical reference range with status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicalRange {
    /// Name of the range (e.g., "Normal", "Low", "High").
    pub label: String,
    /// Lower bound of the range in the metric's unit.
    pub min: f64,
    /// Upper bound of the range in the metric's unit.
    pub max: f64,
    /// Clinical interpretation (e.g., "normal", "elevated", "critical").
    pub status: String,
}
