// SPDX-License-Identifier: AGPL-3.0-or-later
//! Universal spring data adapter for heterogeneous push formats.
//!
//! Springs push visualization data via JSON-RPC using different envelope formats:
//!
//! - **neuralSpring / healthSpring / wetSpring**: `{ "bindings": [ { "channel_type": "...", ... } ] }`
//! - **ludoSpring**: `{ "data": { ... }, "channel": "..." }` with `GameChannelType` semantics
//! - **ecoPrimals/time-series/v1**: `{ "schema": "ecoPrimals/time-series/v1", "series": [ ... ] }`
//!
//! `SpringDataAdapter` normalizes all three formats to `Vec<DataBinding>`.

mod eco_timeseries;
mod game_channel;
mod helpers;

#[cfg(test)]
mod tests;

use crate::data_channel::DataBinding;
use serde::Deserialize;

pub use game_channel::GameChannelType;

/// Recognized envelope formats from springs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpringPayloadFormat {
    /// Standard `{ "bindings": [...] }` (neuralSpring, healthSpring, wetSpring).
    Bindings,
    /// ludoSpring `{ "data": {...}, "channel": "..." }`.
    GameChannel,
    /// ludoSpring game scene `{ "channel_type": "game_scene", "scene": {...} }`.
    GameScene,
    /// Soundscape definition `{ "channel_type": "soundscape", "definition": {...} }`.
    SoundscapePush,
    /// `ecoPrimals/time-series/v1` schema.
    EcoTimeSeries,
    /// Already a raw `DataBinding` array (pass-through).
    Raw,
}

/// Standard bindings envelope: `{ "bindings": [...] }`.
#[derive(Debug, Clone, Deserialize)]
struct BindingsEnvelope {
    bindings: Vec<DataBinding>,
}

/// Universal spring data adapter.
///
/// Accepts any spring push payload and normalizes it to `Vec<DataBinding>`.
pub struct SpringDataAdapter;

impl SpringDataAdapter {
    /// Detect the format of a JSON payload.
    #[must_use]
    pub fn detect_format(value: &serde_json::Value) -> SpringPayloadFormat {
        if let Some(obj) = value.as_object() {
            if obj.get("schema").and_then(|s| s.as_str()) == Some("ecoPrimals/time-series/v1") {
                return SpringPayloadFormat::EcoTimeSeries;
            }
            if obj.contains_key("bindings") {
                return SpringPayloadFormat::Bindings;
            }
            if obj.contains_key("data") && obj.contains_key("channel") {
                return SpringPayloadFormat::GameChannel;
            }
            if obj.get("channel_type").and_then(|v| v.as_str()) == Some("game_scene") {
                return SpringPayloadFormat::GameScene;
            }
            if obj.get("channel_type").and_then(|v| v.as_str()) == Some("soundscape") {
                return SpringPayloadFormat::SoundscapePush;
            }
        }
        if value.is_array() {
            return SpringPayloadFormat::Raw;
        }
        SpringPayloadFormat::Raw
    }

    /// Adapt a raw JSON value to a `Vec<DataBinding>`, auto-detecting the format.
    ///
    /// Takes ownership of the `Value` to avoid deep clones during deserialization.
    ///
    /// # Errors
    ///
    /// Returns an error if the payload cannot be parsed into any recognized format.
    pub fn adapt(value: serde_json::Value) -> Result<Vec<DataBinding>, SpringAdapterError> {
        match Self::detect_format(&value) {
            SpringPayloadFormat::Bindings => Self::adapt_bindings(value),
            SpringPayloadFormat::GameChannel => Self::adapt_game_channel(value),
            SpringPayloadFormat::GameScene => Self::adapt_game_scene(value),
            SpringPayloadFormat::SoundscapePush => Self::adapt_soundscape(value),
            SpringPayloadFormat::EcoTimeSeries => Self::adapt_eco_timeseries(value),
            SpringPayloadFormat::Raw => Self::adapt_raw(value),
        }
    }

    /// Parse standard `{ "bindings": [...] }` envelope.
    fn adapt_bindings(value: serde_json::Value) -> Result<Vec<DataBinding>, SpringAdapterError> {
        let envelope: BindingsEnvelope =
            serde_json::from_value(value).map_err(SpringAdapterError::DeserializeFailed)?;
        Ok(envelope.bindings)
    }

    /// Parse a bare `Vec<DataBinding>` array.
    fn adapt_raw(value: serde_json::Value) -> Result<Vec<DataBinding>, SpringAdapterError> {
        let bindings: Vec<DataBinding> =
            serde_json::from_value(value).map_err(SpringAdapterError::DeserializeFailed)?;
        Ok(bindings)
    }

    /// Parse ludoSpring `{ "data": {...}, "channel": "..." }` and convert to `DataBinding`.
    fn adapt_game_channel(
        value: serde_json::Value,
    ) -> Result<Vec<DataBinding>, SpringAdapterError> {
        game_channel::adapt_game_channel(value)
    }

    /// Parse `ecoPrimals/time-series/v1` and convert each series to `DataBinding::TimeSeries`.
    fn adapt_eco_timeseries(
        value: serde_json::Value,
    ) -> Result<Vec<DataBinding>, SpringAdapterError> {
        eco_timeseries::adapt_eco_timeseries(value)
    }

    /// Parse a game scene push `{ "channel_type": "game_scene", "id": "...", "label": "...", "scene": {...} }`.
    fn adapt_game_scene(
        mut value: serde_json::Value,
    ) -> Result<Vec<DataBinding>, SpringAdapterError> {
        let obj = value.as_object_mut().ok_or_else(|| {
            SpringAdapterError::DeserializeFailed(serde::de::Error::custom("expected object"))
        })?;
        let id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("game_scene")
            .to_string();
        let label = obj
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("Game Scene")
            .to_string();
        let scene = obj.remove("scene").unwrap_or_default();
        Ok(vec![DataBinding::GameScene { id, label, scene }])
    }

    /// Parse a soundscape push `{ "channel_type": "soundscape", "id": "...", "label": "...", "definition": {...} }`.
    fn adapt_soundscape(
        mut value: serde_json::Value,
    ) -> Result<Vec<DataBinding>, SpringAdapterError> {
        let obj = value.as_object_mut().ok_or_else(|| {
            SpringAdapterError::DeserializeFailed(serde::de::Error::custom("expected object"))
        })?;
        let id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("soundscape")
            .to_string();
        let label = obj
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("Soundscape")
            .to_string();
        let definition = obj.remove("definition").unwrap_or_default();
        Ok(vec![DataBinding::Soundscape {
            id,
            label,
            definition,
        }])
    }
}

/// Errors from the spring data adapter.
#[derive(Debug, thiserror::Error)]
pub enum SpringAdapterError {
    /// JSON deserialization failed.
    #[error("failed to deserialize spring payload: {0}")]
    DeserializeFailed(serde_json::Error),

    /// Required field missing from spring payload.
    #[error("missing required field '{field}' in {context}")]
    MissingField {
        /// The field that was expected.
        field: String,
        /// Where the field was expected (e.g., "`game_scene` payload").
        context: String,
    },

    /// Payload format could not be detected.
    #[error("unrecognized spring payload format")]
    UnrecognizedFormat,

    /// Channel type not supported.
    #[error("unsupported channel type: {0}")]
    UnsupportedChannelType(String),
}
