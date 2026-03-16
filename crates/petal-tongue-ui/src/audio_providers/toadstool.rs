// SPDX-License-Identifier: AGPL-3.0-or-later
//! Toadstool audio provider — advanced synthesis via primal HTTP API.

use super::AudioProvider;

#[must_use]
pub fn build_toadstool_play_url(base: &str) -> String {
    format!("{base}/api/v1/audio/play")
}

#[must_use]
pub fn build_toadstool_stop_url(base: &str) -> String {
    format!("{base}/api/v1/audio/stop")
}

#[must_use]
pub fn build_toadstool_synthesize_url(base: &str) -> String {
    format!("{base}/api/v1/audio/synthesize")
}
use bytes::Bytes;
use tracing::{info, warn};

/// Toadstool audio provider (advanced synthesis via primal)
pub struct ToadstoolAudioProvider {
    endpoint: Option<String>,
    available: bool,
}

impl ToadstoolAudioProvider {
    /// Create new Toadstool audio provider (discovers via environment)
    pub fn new() -> Self {
        // Check for toadstool via environment or discovery
        let endpoint = std::env::var("TOADSTOOL_URL").ok();
        let available = endpoint.is_some();

        if available {
            info!("🔊 Toadstool audio provider initialized: {:?}", endpoint);
        } else {
            info!("🔊 Toadstool audio provider not available (set TOADSTOOL_URL)");
        }

        Self {
            endpoint,
            available,
        }
    }

    /// Request audio synthesis from Toadstool
    #[expect(dead_code, reason = "Reserved for future parametric synthesis API")]
    async fn request_synthesis(&self, params: &str) -> Result<Bytes, String> {
        let endpoint = self.endpoint.as_ref().ok_or("Toadstool not configured")?;

        let url = build_toadstool_synthesize_url(endpoint);
        info!("🔊 Requesting audio synthesis from Toadstool: {}", params);

        // Create HTTP client with timeout
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

        #[derive(serde::Serialize)]
        struct SynthesisRequest {
            params: String,
            format: String,
        }

        let request = SynthesisRequest {
            params: params.to_string(),
            format: "wav".to_string(),
        };

        // Make async HTTP POST request
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!(
                "Toadstool returned error: {} ({})",
                response.status(),
                url
            ));
        }

        let audio_bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read audio data: {e}"))?;

        info!(
            "✅ Received {} bytes of audio from Toadstool",
            audio_bytes.len()
        );
        Ok(audio_bytes)
    }
}

impl Default for ToadstoolAudioProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toadstool_provider_default() {
        let provider = ToadstoolAudioProvider::default();
        assert_eq!(provider.name(), "Toadstool Synthesis");
    }

    #[test]
    fn test_toadstool_play_when_unavailable() {
        let provider = ToadstoolAudioProvider::new();
        if !provider.is_available() {
            let result = provider.play("music");
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("not available"));
        }
    }

    #[test]
    fn test_toadstool_stop_when_unavailable_no_panic() {
        let provider = ToadstoolAudioProvider::new();
        provider.stop();
    }

    #[test]
    fn test_toadstool_description() {
        let provider = ToadstoolAudioProvider::new();
        assert!(provider.description().contains("Toadstool"));
    }

    #[test]
    fn test_toadstool_available_sounds_when_unavailable() {
        let provider = ToadstoolAudioProvider::new();
        if !provider.is_available() {
            assert!(provider.available_sounds().is_empty());
        }
    }

    #[test]
    fn test_toadstool_name_constant() {
        let provider = ToadstoolAudioProvider::default();
        assert_eq!(provider.name(), "Toadstool Synthesis");
    }

    #[test]
    fn test_toadstool_description_contains_primal() {
        let provider = ToadstoolAudioProvider::new();
        assert!(provider.description().contains("primal"));
        assert!(provider.description().contains("Toadstool"));
    }

    #[test]
    fn test_build_toadstool_play_url() {
        assert_eq!(
            build_toadstool_play_url("http://localhost:8080"),
            "http://localhost:8080/api/v1/audio/play"
        );
    }

    #[test]
    fn test_build_toadstool_stop_url() {
        assert_eq!(
            build_toadstool_stop_url("http://localhost:8080"),
            "http://localhost:8080/api/v1/audio/stop"
        );
    }

    #[test]
    fn test_build_toadstool_synthesize_url() {
        assert_eq!(
            build_toadstool_synthesize_url("http://localhost:8080"),
            "http://localhost:8080/api/v1/audio/synthesize"
        );
    }
}

impl AudioProvider for ToadstoolAudioProvider {
    fn name(&self) -> &'static str {
        "Toadstool Synthesis"
    }

    fn is_available(&self) -> bool {
        self.available
    }

    fn play(&self, sound_name: &str) -> Result<(), String> {
        if !self.available {
            return Err("Toadstool not available".to_string());
        }

        let endpoint = self.endpoint.clone();
        let sound = sound_name.to_string();

        info!("🔊 Requesting Toadstool synthesis: {}", sound_name);

        // Spawn async task to request synthesis
        tokio::spawn(async move {
            if let Some(ep) = endpoint {
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .build();

                if let Ok(client) = client {
                    let url = build_toadstool_play_url(&ep);

                    #[derive(serde::Serialize)]
                    struct PlayRequest {
                        sound: String,
                    }

                    let request = PlayRequest { sound };

                    match client
                        .post(&url)
                        .header("Content-Type", "application/json")
                        .json(&request)
                        .send()
                        .await
                    {
                        Ok(response) if response.status().is_success() => {
                            info!("✅ Toadstool playing sound");
                        }
                        Ok(response) => {
                            warn!("⚠️ Toadstool returned error: {}", response.status());
                        }
                        Err(e) => {
                            warn!("❌ Failed to request playback: {}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    fn stop(&self) {
        if !self.available {
            return;
        }

        let endpoint = self.endpoint.clone();

        info!("🛑 Sending stop command to Toadstool");

        // Spawn async task to send stop command
        tokio::spawn(async move {
            if let Some(ep) = endpoint {
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(5))
                    .build();

                if let Ok(client) = client {
                    let url = build_toadstool_stop_url(&ep);

                    match client.post(&url).send().await {
                        Ok(response) if response.status().is_success() => {
                            info!("✅ Toadstool stopped playback");
                        }
                        Ok(response) => {
                            warn!("⚠️ Toadstool stop returned: {}", response.status());
                        }
                        Err(e) => {
                            warn!("❌ Failed to send stop command: {}", e);
                        }
                    }
                }
            }
        });
    }

    fn available_sounds(&self) -> Vec<String> {
        if self.available {
            vec![
                "music".to_string(),
                "voice".to_string(),
                "soundscape".to_string(),
                "ambient".to_string(),
                "rhythm".to_string(),
            ]
        } else {
            Vec::new()
        }
    }

    fn description(&self) -> &'static str {
        "Advanced audio synthesis via Toadstool primal. Supports music, voice, and complex soundscapes."
    }
}
