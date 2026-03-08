// SPDX-License-Identifier: AGPL-3.0-only
//! Data channel and clinical range types from healthSpring
//!
//! Schema for visualization of clinical and timeseries data.

use serde::{Deserialize, Serialize};

/// Data channel for health/clinical visualization (healthSpring schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "channel_type")]
pub enum DataChannel {
    #[serde(rename = "timeseries")]
    TimeSeries {
        id: String,
        label: String,
        x_label: String,
        y_label: String,
        unit: String,
        x_values: Vec<f64>,
        y_values: Vec<f64>,
    },
    #[serde(rename = "distribution")]
    Distribution {
        id: String,
        label: String,
        unit: String,
        values: Vec<f64>,
        mean: f64,
        std: f64,
        patient_value: f64,
    },
    #[serde(rename = "bar")]
    Bar {
        id: String,
        label: String,
        categories: Vec<String>,
        values: Vec<f64>,
        unit: String,
    },
    #[serde(rename = "gauge")]
    Gauge {
        id: String,
        label: String,
        value: f64,
        min: f64,
        max: f64,
        unit: String,
        normal_range: [f64; 2],
        warning_range: [f64; 2],
    },
}

/// Clinical reference range with status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicalRange {
    pub label: String,
    pub min: f64,
    pub max: f64,
    pub status: String,
}
