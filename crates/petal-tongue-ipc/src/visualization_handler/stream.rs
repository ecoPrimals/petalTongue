// SPDX-License-Identifier: AGPL-3.0-only
//! Stream update utilities for `DataBinding`.
//!
//! Provides `binding_id` and `apply_operation` for incremental updates
//! to visualization bindings (`TimeSeries` append, Gauge `set_value`, etc.).

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualization_handler::StreamOperation;
    use petal_tongue_core::DataBinding;

    fn ts(id: &str, x: Vec<f64>, y: Vec<f64>) -> DataBinding {
        DataBinding::TimeSeries {
            id: id.into(),
            label: String::new(),
            x_label: String::new(),
            y_label: String::new(),
            unit: String::new(),
            x_values: x,
            y_values: y,
        }
    }

    fn gauge(id: &str, value: f64) -> DataBinding {
        DataBinding::Gauge {
            id: id.into(),
            label: String::new(),
            value,
            min: 0.0,
            max: 100.0,
            unit: String::new(),
            normal_range: [0.0, 100.0],
            warning_range: [0.0, 100.0],
        }
    }

    fn spectrum(id: &str, f: Vec<f64>, a: Vec<f64>) -> DataBinding {
        DataBinding::Spectrum {
            id: id.into(),
            label: String::new(),
            frequencies: f,
            amplitudes: a,
            unit: String::new(),
        }
    }

    #[test]
    fn binding_id_time_series() {
        let b = ts("ts1", vec![], vec![]);
        assert_eq!(binding_id(&b), "ts1");
    }

    #[test]
    fn binding_id_gauge() {
        let b = gauge("g1", 42.0);
        assert_eq!(binding_id(&b), "g1");
    }

    #[test]
    fn binding_id_spectrum() {
        let b = spectrum("sp1", vec![], vec![]);
        assert_eq!(binding_id(&b), "sp1");
    }

    #[test]
    fn apply_operation_timeseries_append() {
        let mut b = ts("ts1", vec![1.0], vec![10.0]);
        apply_operation(
            &mut b,
            &StreamOperation::Append {
                x_values: vec![2.0, 3.0],
                y_values: vec![20.0, 30.0],
            },
        );
        let DataBinding::TimeSeries {
            x_values, y_values, ..
        } = &b
        else {
            panic!("expected TimeSeries");
        };
        assert_eq!(x_values, &[1.0, 2.0, 3.0]);
        assert_eq!(y_values, &[10.0, 20.0, 30.0]);
    }

    #[test]
    fn apply_operation_gauge_set_value() {
        let mut b = gauge("g1", 0.0);
        apply_operation(&mut b, &StreamOperation::SetValue { value: 99.5 });
        let DataBinding::Gauge { value, .. } = &b else {
            panic!("expected Gauge");
        };
        assert!((*value - 99.5).abs() < f64::EPSILON);
    }

    #[test]
    fn apply_operation_replace() {
        let mut b = gauge("g1", 0.0);
        let new_b = ts("ts2", vec![1.0], vec![2.0]);
        apply_operation(&mut b, &StreamOperation::Replace { binding: new_b });
        assert!(matches!(&b, DataBinding::TimeSeries { id, .. } if id == "ts2"));
    }

    #[test]
    fn apply_operation_spectrum_append() {
        let mut b = spectrum("sp1", vec![100.0], vec![0.5]);
        apply_operation(
            &mut b,
            &StreamOperation::Append {
                x_values: vec![200.0],
                y_values: vec![0.8],
            },
        );
        let DataBinding::Spectrum {
            frequencies,
            amplitudes,
            ..
        } = &b
        else {
            panic!("expected Spectrum");
        };
        assert_eq!(frequencies, &[100.0, 200.0]);
        assert_eq!(amplitudes, &[0.5, 0.8]);
    }

    #[test]
    fn apply_operation_mismatched_no_panic() {
        let mut b = gauge("g1", 1.0);
        apply_operation(
            &mut b,
            &StreamOperation::Append {
                x_values: vec![1.0],
                y_values: vec![2.0],
            },
        );
        let DataBinding::Gauge { value, .. } = &b else {
            panic!("expected Gauge");
        };
        assert!((*value - 1.0).abs() < f64::EPSILON);
    }
}
