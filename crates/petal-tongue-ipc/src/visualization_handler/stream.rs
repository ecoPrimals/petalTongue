// SPDX-License-Identifier: AGPL-3.0-only
//! Stream update utilities for DataBinding.
//!
//! Provides `binding_id` and `apply_operation` for incremental updates
//! to visualization bindings (TimeSeries append, Gauge set_value, etc.).

use petal_tongue_core::DataBinding;
use tracing::warn;

use super::types::StreamOperation;

/// Extract the `id` field from any `DataBinding` variant
#[must_use]
pub fn binding_id(binding: &DataBinding) -> &str {
    match binding {
        DataBinding::TimeSeries { id, .. }
        | DataBinding::Distribution { id, .. }
        | DataBinding::Bar { id, .. }
        | DataBinding::Gauge { id, .. }
        | DataBinding::Heatmap { id, .. }
        | DataBinding::Scatter3D { id, .. }
        | DataBinding::Scatter { id, .. }
        | DataBinding::FieldMap { id, .. }
        | DataBinding::Spectrum { id, .. } => id,
    }
}

/// Apply a stream operation to a binding in place
pub fn apply_operation(binding: &mut DataBinding, operation: &StreamOperation) {
    match (binding, operation) {
        (
            DataBinding::TimeSeries {
                x_values, y_values, ..
            },
            StreamOperation::Append {
                x_values: new_x,
                y_values: new_y,
            },
        ) => {
            x_values.extend(new_x.iter().copied());
            y_values.extend(new_y.iter().copied());
        }
        (
            DataBinding::Spectrum {
                frequencies,
                amplitudes,
                ..
            },
            StreamOperation::Append {
                x_values: new_freq,
                y_values: new_amp,
            },
        ) => {
            frequencies.extend(new_freq.iter().copied());
            amplitudes.extend(new_amp.iter().copied());
        }
        (DataBinding::Gauge { value, .. }, StreamOperation::SetValue { value: new_val }) => {
            *value = *new_val;
        }
        (b, StreamOperation::Replace { binding: new_b }) => {
            *b = new_b.clone();
        }
        _ => {
            warn!("Mismatched stream operation: operation not applicable to this binding type");
        }
    }
}
