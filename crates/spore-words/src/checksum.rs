//! CRC-8 terminal checksum (IETF draft-saywhere-geocoding section 8.3).

/// 32-word checksum vocabulary (from IETF spec).
const CHECKSUM_WORDS: [&str; 32] = [
    "arch", "bell", "cape", "dawn", "echo", "fern", "glen", "haze", "isle", "jade", "keel",
    "lake", "mist", "nook", "opal", "pine", "quay", "reef", "sail", "tide", "vale", "wave",
    "yard", "zeal", "apex", "brew", "clad", "dusk", "emit", "flux", "gale", "helm",
];

/// Compute CRC-8/CCITT checksum (polynomial `0x07`, init `0x00`).
pub fn crc8(data: &[u8]) -> u8 {
    let mut crc = 0_u8;
    for &byte in data {
        crc ^= byte;
        for _ in 0..8 {
            if crc & 0x80 != 0 {
                crc = (crc << 1) ^ 0x07;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// Compute terminal checksum word for a geohash string.
pub fn checksum_word(geohash: &str) -> &'static str {
    let index = usize::from(crc8(geohash.as_bytes()) % 32);
    CHECKSUM_WORDS[index]
}

/// Validate a terminal checksum word against a geohash.
pub fn validate_checksum(geohash: &str, word: &str) -> bool {
    checksum_word(geohash) == word
}
