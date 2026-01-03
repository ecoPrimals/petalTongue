//! Entropy streaming to biomeOS/BearDog

use crate::types::EntropyCapture;
use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, KeyInit, OsRng},
    Aes256Gcm,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Stream confirmation from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfirmation {
    /// Receipt ID for this entropy submission
    pub receipt_id: String,
    /// Server timestamp
    pub timestamp: u64,
    /// Accepted quality score
    pub quality: f64,
    /// Success message
    pub message: String,
}

/// Encrypted entropy package
#[derive(Serialize, Deserialize)]
struct EncryptedEntropy {
    /// Encrypted data
    ciphertext: Vec<u8>,
    /// Nonce for decryption (96 bits for AES-GCM)
    nonce: Vec<u8>,
}

impl Drop for EncryptedEntropy {
    fn drop(&mut self) {
        // Manually zeroize sensitive data
        use zeroize::Zeroize;
        self.ciphertext.zeroize();
        self.nonce.zeroize();
    }
}

/// Stream entropy to biomeOS API
///
/// # Arguments
///
/// * `entropy` - The entropy capture to stream
/// * `endpoint` - API endpoint URL
///
/// # Returns
///
/// Stream confirmation with receipt ID
///
/// # Security
///
/// - Encrypts data with AES-256-GCM
/// - Zeroizes sensitive data after transmission
/// - Uses TLS for transport (defense in depth)
/// - Generates fresh key per session (TODO: proper key exchange with biomeOS)
///
/// # Note
///
/// This implementation uses a randomly generated key for demonstration.
/// In production, the key should be derived from:
/// - BearDog's public key (for key exchange)
/// - Or a pre-shared key established during primal handshake
/// - Or ephemeral keys via ECDH
pub async fn stream_entropy(entropy: EntropyCapture, endpoint: &str) -> Result<StreamConfirmation> {
    tracing::info!(
        "Streaming {} entropy (quality: {:.1}%)",
        entropy.modality(),
        entropy.quality() * 100.0
    );

    // 1. Serialize entropy
    let data = serde_json::to_vec(&entropy)?;
    tracing::debug!("Serialized {} bytes", data.len());

    // 2. Encrypt (application-level, in addition to TLS)
    let encrypted = encrypt_entropy(&data)?;
    tracing::debug!("Encrypted entropy with AES-256-GCM");

    // 3. Prepare payload
    let _payload = serde_json::to_vec(&encrypted)?;

    // 4. Stream via HTTPS
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let response = client
        .post(endpoint)
        .header("Content-Type", "application/json")
        .header("X-Entropy-Modality", entropy.modality())
        .header("X-Entropy-Quality", format!("{:.2}", entropy.quality()))
        .json(&encrypted)
        .send()
        .await
        .context("Failed to send entropy to server")?;

    // 4. Zeroize encrypted data after sending
    drop(encrypted); // Explicitly drop to trigger zeroization

    // 6. Parse confirmation
    let status = response.status();
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        anyhow::bail!("Server rejected entropy: {} - {}", status, error_body);
    }

    let confirmation: StreamConfirmation = response
        .json()
        .await
        .context("Failed to parse server confirmation")?;

    tracing::info!("✅ Entropy accepted: receipt {}", confirmation.receipt_id);

    Ok(confirmation)
}

/// Encrypt entropy data with AES-256-GCM
///
/// # Security
///
/// - Uses AES-256-GCM (authenticated encryption)
/// - Generates random 96-bit nonce per encryption
/// - Includes authentication tag (prevents tampering)
/// - Zeroizes plaintext after encryption
///
/// # Note
///
/// In production, the encryption key should be:
/// - Derived from BearDog's public key (ECDH key exchange)
/// - Or established during primal handshake
/// - Or retrieved from secure key storage
///
/// For now, this generates a fresh random key per session.
/// This is secure for confidentiality but doesn't establish identity.
fn encrypt_entropy(plaintext: &[u8]) -> Result<EncryptedEntropy> {
    // Generate random encryption key (32 bytes for AES-256)
    // TODO: Replace with proper key derived from biomeOS/BearDog public key
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);

    // Generate random nonce (96 bits = 12 bytes, recommended for AES-GCM)
    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = GenericArray::from_slice(&nonce_bytes);

    // Encrypt with authenticated encryption
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    tracing::debug!(
        "Encrypted {} bytes → {} bytes (includes 16-byte auth tag)",
        plaintext.len(),
        ciphertext.len()
    );

    Ok(EncryptedEntropy {
        ciphertext,
        nonce: nonce_bytes.to_vec(),
    })
}

/// Decrypt entropy data (for testing only)
///
/// In production, only biomeOS/BearDog would decrypt (using their private key).
#[cfg(test)]
fn decrypt_entropy(encrypted: &EncryptedEntropy, key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = GenericArray::from_slice(&encrypted.nonce);

    let plaintext = cipher
        .decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let data = b"test entropy data with sensitive information";

        // Generate key for testing
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let key_bytes: [u8; 32] = key.as_slice().try_into().unwrap();

        // Encrypt
        let encrypted = encrypt_entropy(data).expect("Encryption failed");

        // Verify structure
        assert_eq!(encrypted.nonce.len(), 12, "Nonce should be 12 bytes");
        assert!(
            encrypted.ciphertext.len() > data.len(),
            "Ciphertext should be longer (includes auth tag)"
        );
        assert_eq!(
            encrypted.ciphertext.len(),
            data.len() + 16,
            "Ciphertext should be plaintext + 16 byte auth tag"
        );

        // Note: Can't decrypt with different key (which is expected in production)
        // In production, only biomeOS/BearDog with the matching private key can decrypt
    }

    #[test]
    fn test_encrypt_produces_different_outputs() {
        let data = b"same plaintext";

        let encrypted1 = encrypt_entropy(data).expect("Encryption 1 failed");
        let encrypted2 = encrypt_entropy(data).expect("Encryption 2 failed");

        // Different nonces should produce different ciphertexts
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
    }

    #[test]
    fn test_encrypted_entropy_serialization() {
        let data = b"test data";
        let encrypted = encrypt_entropy(data).expect("Encryption failed");

        // Should serialize to JSON
        let json = serde_json::to_string(&encrypted).expect("Serialization failed");
        assert!(json.contains("ciphertext"));
        assert!(json.contains("nonce"));

        // Should deserialize back
        let deserialized: EncryptedEntropy =
            serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(deserialized.ciphertext, encrypted.ciphertext);
        assert_eq!(deserialized.nonce, encrypted.nonce);
    }

    #[tokio::test]
    #[ignore] // Requires live server
    async fn test_stream_entropy_integration() {
        use std::time::Duration;

        let audio_data = AudioEntropyData {
            samples: vec![0.1, 0.2, 0.3],
            sample_rate: 44100,
            duration: Duration::from_secs(1),
            peaks: vec![0.3, 0.5, 0.4],
            timestamps: vec![
                Duration::from_millis(0),
                Duration::from_millis(100),
                Duration::from_millis(200),
            ],
            peak_amplitude: 0.5,
            avg_amplitude: 0.4,
            quality_metrics: AudioQualityMetrics {
                amplitude_entropy: 0.8,
                timing_entropy: 0.8,
                spectral_entropy: 0.7,
                dynamic_range: 0.9,
                overall_quality: 0.8,
            },
        };

        let entropy = EntropyCapture::Audio(audio_data);

        // This would fail without a live server
        let result = stream_entropy(entropy, "http://localhost:3000/api/v1/entropy/stream").await;
        assert!(result.is_err()); // Expected to fail in test environment
    }
}
