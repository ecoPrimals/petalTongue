// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scene graph integrity signing using BLAKE3 keyed hash.
//!
//! Implements the `visualization` purpose key from the NUCLEUS Two-Tier Crypto Model.
//! When a purpose key is available (via `PETALTONGUE_SCENE_KEY` env var or BearDog
//! `secrets.retrieve("visualization")`), scene pushes carry a BLAKE3 keyed-hash
//! signature that compositions can verify to ensure authentic UI updates.
//!
//! Key sources (priority order):
//! 1. `PETALTONGUE_SCENE_KEY` — hex-encoded 32-byte key from composition bootstrap
//! 2. Future: BearDog `crypto.sign` delegation via IPC
//! 3. If no key is available, signing is skipped (unsigned scenes are valid but unverified)

/// A scene signer that produces BLAKE3 keyed-hash signatures.
///
/// Thread-safe: the key is immutable after construction.
#[derive(Clone)]
pub struct SceneSigner {
    key: Option<[u8; blake3::KEY_LEN]>,
}

impl SceneSigner {
    /// Create a signer from the environment.
    ///
    /// Reads `PETALTONGUE_SCENE_KEY` (hex-encoded, 64 hex chars = 32 bytes).
    /// Returns a no-op signer if the variable is absent or malformed.
    #[must_use]
    pub fn from_env() -> Self {
        let key = std::env::var("PETALTONGUE_SCENE_KEY")
            .ok()
            .and_then(|hex| Self::decode_hex_key(&hex));

        if key.is_some() {
            tracing::info!("scene signer: visualization purpose key loaded");
        } else {
            tracing::debug!("scene signer: no PETALTONGUE_SCENE_KEY — scenes will be unsigned");
        }

        Self { key }
    }

    /// Create a signer with an explicit key (for testing or BearDog delegation).
    #[must_use]
    pub const fn with_key(key: [u8; blake3::KEY_LEN]) -> Self {
        Self { key: Some(key) }
    }

    /// Create a no-op signer (no key available).
    #[must_use]
    pub const fn unsigned() -> Self {
        Self { key: None }
    }

    /// Whether this signer has a key loaded.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.key.is_some()
    }

    /// Sign a scene payload, returning a hex-encoded BLAKE3 keyed hash.
    ///
    /// Returns `None` if no key is loaded.
    #[must_use]
    pub fn sign(&self, payload: &[u8]) -> Option<String> {
        let key = self.key.as_ref()?;
        let hash = blake3::keyed_hash(key, payload);
        Some(hash.to_hex().to_string())
    }

    /// Verify a scene payload against a hex-encoded signature.
    ///
    /// Returns `false` if no key is loaded or the signature doesn't match.
    #[must_use]
    pub fn verify(&self, payload: &[u8], signature_hex: &str) -> bool {
        let Some(ref key) = self.key else {
            return false;
        };
        let expected = blake3::keyed_hash(key, payload);
        expected.to_hex().as_str() == signature_hex
    }

    /// Decode a hex-encoded 32-byte key.
    fn decode_hex_key(hex: &str) -> Option<[u8; blake3::KEY_LEN]> {
        if hex.len() != blake3::KEY_LEN * 2 {
            tracing::warn!(
                "scene signer: PETALTONGUE_SCENE_KEY must be {} hex chars, got {}",
                blake3::KEY_LEN * 2,
                hex.len()
            );
            return None;
        }
        let mut key = [0u8; blake3::KEY_LEN];
        for (i, byte) in key.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
        }
        Some(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; blake3::KEY_LEN] {
        let mut key = [0u8; blake3::KEY_LEN];
        for (i, b) in key.iter_mut().enumerate() {
            #[expect(clippy::cast_possible_truncation, reason = "test key bytes")]
            {
                *b = (i + 1) as u8;
            }
        }
        key
    }

    fn test_key_hex() -> String {
        test_key().iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn signer_with_key_signs_and_verifies() {
        let signer = SceneSigner::with_key(test_key());
        assert!(signer.is_active());
        let payload = b"test scene payload";
        let sig = signer.sign(payload).expect("should sign");
        assert_eq!(sig.len(), 64);
        assert!(signer.verify(payload, &sig));
    }

    #[test]
    fn signer_rejects_tampered_payload() {
        let signer = SceneSigner::with_key(test_key());
        let sig = signer.sign(b"original").expect("should sign");
        assert!(!signer.verify(b"tampered", &sig));
    }

    #[test]
    fn signer_rejects_wrong_signature() {
        let signer = SceneSigner::with_key(test_key());
        assert!(!signer.verify(b"payload", "0000000000000000000000000000000000000000000000000000000000000000"));
    }

    #[test]
    fn unsigned_signer_returns_none() {
        let signer = SceneSigner::unsigned();
        assert!(!signer.is_active());
        assert!(signer.sign(b"data").is_none());
        assert!(!signer.verify(b"data", "anything"));
    }

    #[test]
    fn decode_hex_key_valid() {
        let hex = test_key_hex();
        let decoded = SceneSigner::decode_hex_key(&hex);
        assert_eq!(decoded, Some(test_key()));
    }

    #[test]
    fn decode_hex_key_wrong_length() {
        assert!(SceneSigner::decode_hex_key("abcd").is_none());
    }

    #[test]
    fn decode_hex_key_invalid_hex() {
        let bad = "zz".repeat(32);
        assert!(SceneSigner::decode_hex_key(&bad).is_none());
    }

    #[test]
    fn decode_then_construct_round_trips() {
        let hex = test_key_hex();
        let key = SceneSigner::decode_hex_key(&hex).expect("should decode");
        let signer = SceneSigner::with_key(key);
        let sig = signer.sign(b"hello").expect("should sign");
        assert!(signer.verify(b"hello", &sig));
    }

    #[test]
    fn different_keys_produce_different_signatures() {
        let key_a = SceneSigner::with_key([1u8; blake3::KEY_LEN]);
        let key_b = SceneSigner::with_key([2u8; blake3::KEY_LEN]);
        let payload = b"same payload";
        let sig_a = key_a.sign(payload).expect("sign a");
        let sig_b = key_b.sign(payload).expect("sign b");
        assert_ne!(sig_a, sig_b);
        assert!(!key_a.verify(payload, &sig_b));
        assert!(!key_b.verify(payload, &sig_a));
    }
}
