// SPDX-License-Identifier: AGPL-3.0-or-later

/// Typed error for sensor polling failures.
#[derive(Debug, thiserror::Error)]
pub enum SensorError {
    /// The sensor device is not currently available.
    #[error("sensor unavailable: {0}")]
    Unavailable(String),

    /// An I/O error occurred during polling.
    #[error("sensor I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The sensor returned malformed or unparseable data.
    #[error("sensor data error: {0}")]
    DataFormat(String),

    /// A timeout occurred waiting for sensor response.
    #[error("sensor timeout: {0}")]
    Timeout(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn sensor_error_display_strings() {
        assert_eq!(
            SensorError::Unavailable("camera".into()).to_string(),
            "sensor unavailable: camera"
        );
        assert_eq!(
            SensorError::Io(io::Error::new(io::ErrorKind::NotFound, "device")).to_string(),
            "sensor I/O error: device"
        );
        assert_eq!(
            SensorError::DataFormat("bad frame".into()).to_string(),
            "sensor data error: bad frame"
        );
        assert_eq!(
            SensorError::Timeout("poll".into()).to_string(),
            "sensor timeout: poll"
        );
    }
}
