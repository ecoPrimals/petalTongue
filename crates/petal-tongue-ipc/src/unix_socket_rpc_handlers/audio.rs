// SPDX-License-Identifier: AGPL-3.0-or-later
//! Handler for `audio.synthesize`: accepts a soundscape definition and returns
//! synthesized stereo PCM as base64-encoded WAV or raw sample metadata.

use super::RpcHandlers;
use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse, error_codes};
use petal_tongue_scene::soundscape::{Soundscape, synthesize_soundscape};
use serde_json::json;

/// Handle `audio.synthesize`: synthesize a soundscape definition to stereo PCM.
///
/// Params:
/// - `definition`: Soundscape JSON (layers, duration, master_amplitude, etc.)
/// - `format` (optional): `"metadata"` (default) returns sample stats,
///   `"wav_base64"` returns full WAV as base64.
///
/// Returns sample_rate, channels, duration_secs, num_samples, and optionally wav_base64.
pub fn handle_audio_synthesize(
    _handlers: &RpcHandlers,
    mut req: JsonRpcRequest,
) -> JsonRpcResponse {
    let format_str = req
        .params
        .get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("metadata")
        .to_string();

    let definition = match req
        .params
        .as_object_mut()
        .and_then(|m| m.remove("definition"))
    {
        Some(v) if !v.is_null() => v,
        _ => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INVALID_PARAMS,
                "Missing required 'definition' parameter".to_string(),
            );
        }
    };

    let soundscape: Soundscape = match serde_json::from_value(definition) {
        Ok(s) => s,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INVALID_PARAMS,
                format!("Invalid soundscape definition: {e}"),
            );
        }
    };

    let samples = synthesize_soundscape(&soundscape);
    let format = format_str.as_str();

    let mut result = json!({
        "sample_rate": samples.sample_rate,
        "channels": 2,
        "duration_secs": samples.duration_secs(),
        "num_samples": samples.left.len(),
        "layers": soundscape.layers.len(),
    });

    if format == "wav_base64" {
        match encode_wav_base64(&samples) {
            Ok(b64) => {
                result["wav_base64"] = serde_json::Value::String(b64);
            }
            Err(e) => {
                return JsonRpcResponse::error(
                    req.id,
                    error_codes::INTERNAL_ERROR,
                    format!("WAV encoding failed: {e}"),
                );
            }
        }
    }

    JsonRpcResponse::success(req.id, result)
}

fn encode_wav_base64(
    samples: &petal_tongue_scene::soundscape::StereoSamples,
) -> Result<String, String> {
    use std::io::Cursor;

    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: samples.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut buf = Cursor::new(Vec::new());
    let mut writer =
        hound::WavWriter::new(&mut buf, spec).map_err(|e| format!("WAV writer: {e}"))?;

    let interleaved = samples.interleaved();
    for s in &interleaved {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "audio: f32 [-1,1] → i16 intentional"
        )]
        let sample_i16 = (s.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16;
        writer
            .write_sample(sample_i16)
            .map_err(|e| format!("WAV sample: {e}"))?;
    }
    writer
        .finalize()
        .map_err(|e| format!("WAV finalize: {e}"))?;

    use base64::Engine;
    let wav_bytes = buf.into_inner();
    Ok(base64::engine::general_purpose::STANDARD.encode(&wav_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_handlers() -> RpcHandlers {
        use crate::visualization_handler::VisualizationState;
        use petal_tongue_core::graph_engine::GraphEngine;
        use std::sync::{Arc, RwLock};
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let viz_state = Arc::new(RwLock::new(VisualizationState::new()));
        RpcHandlers::new(graph, "test".to_string(), viz_state)
    }

    #[test]
    fn synthesize_missing_definition() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("audio.synthesize", json!({}), json!(1));
        let resp = handle_audio_synthesize(&h, req);
        assert!(resp.error.is_some());
    }

    #[test]
    fn synthesize_returns_metadata() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "audio.synthesize",
            json!({
                "definition": {
                    "name": "test",
                    "duration_secs": 1.0,
                    "layers": [{
                        "id": "tone",
                        "waveform": "sine",
                        "frequency": 440.0,
                        "amplitude": 0.5,
                        "duration_secs": 1.0
                    }]
                }
            }),
            json!(1),
        );
        let resp = handle_audio_synthesize(&h, req);
        assert!(resp.error.is_none());
        let r = resp.result.expect("result");
        assert_eq!(r["channels"], 2);
        assert!(r["num_samples"].as_u64().unwrap() > 0);
        assert!(r["duration_secs"].as_f64().unwrap() > 0.0);
    }

    #[test]
    fn synthesize_wav_base64() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "audio.synthesize",
            json!({
                "definition": {
                    "name": "short",
                    "duration_secs": 0.1,
                    "layers": [{
                        "id": "blip",
                        "waveform": "square",
                        "frequency": 880.0,
                        "amplitude": 0.3,
                        "duration_secs": 0.1
                    }]
                },
                "format": "wav_base64"
            }),
            json!(1),
        );
        let resp = handle_audio_synthesize(&h, req);
        assert!(resp.error.is_none());
        let r = resp.result.expect("result");
        let b64 = r["wav_base64"].as_str().expect("wav_base64 string");
        assert!(!b64.is_empty());
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .expect("valid base64");
        assert!(decoded.len() > 44, "WAV header + data");
    }

    #[test]
    fn synthesize_invalid_definition() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "audio.synthesize",
            json!({"definition": "not an object"}),
            json!(1),
        );
        let resp = handle_audio_synthesize(&h, req);
        assert!(resp.error.is_some());
    }
}
