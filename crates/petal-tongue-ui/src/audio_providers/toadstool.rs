// SPDX-License-Identifier: AGPL-3.0-only
//! Toadstool audio provider — advanced synthesis via primal HTTP API.

use super::AudioProvider;
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
    #[allow(dead_code)]
    async fn request_synthesis(&self, params: &str) -> Result<Vec<u8>, String> {
        let endpoint = self.endpoint.as_ref().ok_or("Toadstool not configured")?;

        let url = format!("{endpoint}/api/v1/audio/synthesize");
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

        // Get audio bytes
        let audio_bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read audio data: {e}"))?;

        info!(
            "✅ Received {} bytes of audio from Toadstool",
            audio_bytes.len()
        );
        Ok(audio_bytes.to_vec())
    }
}

impl Default for ToadstoolAudioProvider {
    fn default() -> Self {
        Self::new()
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
                    let url = format!("{ep}/api/v1/audio/play");

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
                    let url = format!("{ep}/api/v1/audio/stop");

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
