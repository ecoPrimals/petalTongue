// SPDX-License-Identifier: AGPL-3.0-or-later
//! `ecoPrimals/time-series/v1` envelope parsing.

use crate::data_channel::DataBinding;
use serde::Deserialize;

use super::SpringAdapterError;

/// `ecoPrimals/time-series/v1` envelope.
#[derive(Debug, Clone, Deserialize)]
struct EcoTimeSeriesEnvelope {
    schema: String,
    series: Vec<EcoTimeSeriesEntry>,
}

/// Single entry in `ecoPrimals/time-series/v1`.
#[derive(Debug, Clone, Deserialize)]
struct EcoTimeSeriesEntry {
    id: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    unit: String,
    #[serde(default)]
    x_label: String,
    #[serde(default)]
    y_label: String,
    timestamps: Vec<f64>,
    values: Vec<f64>,
}

/// Parse `ecoPrimals/time-series/v1` and convert each series to `DataBinding::TimeSeries`.
pub(super) fn adapt_eco_timeseries(
    value: serde_json::Value,
) -> Result<Vec<DataBinding>, SpringAdapterError> {
    let envelope: EcoTimeSeriesEnvelope =
        serde_json::from_value(value).map_err(SpringAdapterError::DeserializeFailed)?;
    if envelope.schema != "ecoPrimals/time-series/v1" {
        return Err(SpringAdapterError::UnrecognizedFormat);
    }
    let bindings = envelope
        .series
        .into_iter()
        .map(|entry| DataBinding::TimeSeries {
            id: entry.id,
            label: entry.label,
            x_label: if entry.x_label.is_empty() {
                "Time".to_string()
            } else {
                entry.x_label
            },
            y_label: entry.y_label,
            unit: entry.unit,
            x_values: entry.timestamps,
            y_values: entry.values,
        })
        .collect();
    Ok(bindings)
}
