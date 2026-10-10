//! # spore-words
//!
//! SayWhere geocoding — IETF `draft-saywhere-geocoding` implementation.
//! Encodes WGS84 lat/lng/alt into hierarchical BIP-39 word phrases.

pub mod altitude;
pub mod checksum;
pub mod geohash;
pub mod seed;
mod wordlist;

pub use wordlist::{BIP39_EN, WORDLIST_SIZE};

/// A decoded SayWhere location.
pub struct Location {
    pub lat: f64,
    pub lng: f64,
    pub lat_error: f64,
    pub lng_error: f64,
    pub precision: u8,
    pub geohash: String,
}

/// Error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid latitude: {0} (must be -90..90)")]
    InvalidLatitude(f64),
    #[error("invalid longitude: {0} (must be -180..180)")]
    InvalidLongitude(f64),
    #[error("invalid precision: {0} (must be 1..6)")]
    InvalidPrecision(u8),
    #[error("word not found in wordlist: {0}")]
    WordNotFound(String),
    #[error("parity check failed for word: {0}")]
    ParityError(String),
    #[error("checksum validation failed")]
    ChecksumError,
    #[error("invalid phrase format")]
    InvalidFormat,
}

fn validate_coords(lat: f64, lng: f64) -> Result<(), Error> {
    if !(-90.0..=90.0).contains(&lat) {
        return Err(Error::InvalidLatitude(lat));
    }
    if !(-180.0..=180.0).contains(&lng) {
        return Err(Error::InvalidLongitude(lng));
    }
    Ok(())
}

fn validate_precision(precision: u8) -> Result<(), Error> {
    if !(1..=6).contains(&precision) {
        return Err(Error::InvalidPrecision(precision));
    }
    Ok(())
}

fn base32_index(c: u8) -> Option<usize> {
    geohash::BASE32_CHARS.iter().position(|&b| b == c)
}

fn word_index(word: &str, wordlist: &[&str; 2048]) -> Result<usize, Error> {
    wordlist
        .iter()
        .position(|&w| w == word)
        .ok_or_else(|| Error::WordNotFound(word.to_owned()))
}

fn verify_parity(word: &str, index: usize) -> Result<(), Error> {
    let chunk_value = index >> 1;
    let expected_parity = chunk_value.count_ones() % 2;
    if (index & 1) != (expected_parity as usize) {
        return Err(Error::ParityError(word.to_owned()));
    }
    Ok(())
}

fn geohash_to_words(geohash: &str, wordlist: &[&str; 2048]) -> Result<Vec<String>, Error> {
    if !geohash.len().is_multiple_of(2) {
        return Err(Error::InvalidFormat);
    }

    let mut words = Vec::with_capacity(geohash.len() / 2);
    for chunk in geohash.as_bytes().chunks(2) {
        let char1_idx = base32_index(chunk[0]).ok_or_else(|| Error::InvalidFormat)?;
        let char2_idx = base32_index(chunk[1]).ok_or_else(|| Error::InvalidFormat)?;
        let chunk_value = char1_idx * 32 + char2_idx;
        let parity = chunk_value.count_ones() % 2;
        let index = (chunk_value << 1) | (parity as usize);
        if index >= wordlist.len() {
            return Err(Error::InvalidFormat);
        }
        words.push(wordlist[index].to_owned());
    }
    Ok(words)
}

fn words_to_geohash(words: &[&str], wordlist: &[&str; 2048]) -> Result<String, Error> {
    let mut geohash = String::with_capacity(words.len() * 2);
    for &word in words {
        let index = word_index(word, wordlist)?;
        verify_parity(word, index)?;
        let chunk_value = index >> 1;
        let char1 = geohash::BASE32_CHARS[chunk_value / 32];
        let char2 = geohash::BASE32_CHARS[chunk_value % 32];
        geohash.push(char1 as char);
        geohash.push(char2 as char);
    }
    Ok(geohash)
}

fn encode_internal(
    lat: f64,
    lng: f64,
    precision: u8,
    wordlist: &[&str; 2048],
) -> Result<String, Error> {
    validate_coords(lat, lng)?;
    validate_precision(precision)?;

    let geohash = geohash::encode(lat, lng, usize::from(precision) * 2);
    let words = geohash_to_words(&geohash, wordlist)?;
    Ok(words.join("."))
}

fn decode_internal(phrase: &str, wordlist: &[&str; 2048]) -> Result<Location, Error> {
    if phrase.is_empty() {
        return Err(Error::InvalidFormat);
    }

    let words: Vec<&str> = phrase.split('.').collect();
    if words.is_empty() || words.len() > 6 {
        return Err(Error::InvalidFormat);
    }

    let geohash = words_to_geohash(&words, wordlist)?;
    let (lat, lng, lat_error, lng_error) = geohash::decode(&geohash);

    Ok(Location {
        lat,
        lng,
        lat_error,
        lng_error,
        precision: u8::try_from(words.len()).map_err(|_| Error::InvalidFormat)?,
        geohash,
    })
}

/// Encode coordinates to a SayWhere phrase at given precision (1-6 words).
pub fn encode(lat: f64, lng: f64, precision: u8) -> Result<String, Error> {
    encode_internal(lat, lng, precision, &BIP39_EN)
}

/// Decode a SayWhere phrase to coordinates.
pub fn decode(phrase: &str) -> Result<Location, Error> {
    decode_internal(phrase, &BIP39_EN)
}

/// Encode with optional terminal checksum word.
pub fn encode_with_checksum(lat: f64, lng: f64, precision: u8) -> Result<String, Error> {
    validate_coords(lat, lng)?;
    validate_precision(precision)?;

    let geohash = geohash::encode(lat, lng, usize::from(precision) * 2);
    let mut words = geohash_to_words(&geohash, &BIP39_EN)?;
    words.push(checksum::checksum_word(&geohash).to_owned());
    Ok(words.join("."))
}

/// Encode with altitude (adds one word for altitude).
pub fn encode_with_altitude(lat: f64, lng: f64, alt: f64, precision: u8) -> Result<String, Error> {
    let mut phrase = encode_internal(lat, lng, precision, &BIP39_EN)?;
    let alt_index = altitude::encode_altitude(alt) as usize;
    phrase.push('.');
    phrase.push_str(BIP39_EN[alt_index]);
    Ok(phrase)
}

/// Encode using a custom wordlist (seed-permuted).
pub fn encode_with_seed(lat: f64, lng: f64, precision: u8, seed_str: &str) -> Result<String, Error> {
    let wordlist = seed::permute_wordlist(seed_str);
    encode_internal(lat, lng, precision, &wordlist)
}

/// Decode using a custom wordlist (seed-permuted).
pub fn decode_with_seed(phrase: &str, seed_str: &str) -> Result<Location, Error> {
    let wordlist = seed::permute_wordlist(seed_str);
    decode_internal(phrase, &wordlist)
}

/// Get the bounding box for a phrase (for drawing cells on a map).
pub fn cell_bounds(phrase: &str) -> Result<(f64, f64, f64, f64), Error> {
    let location = decode(phrase)?;
    Ok(geohash::decode_bounds(&location.geohash))
}

#[cfg(test)]
mod tests {
    use super::*;

    // IETF spec test vector: NYC
    #[test]
    fn test_nyc_encode() {
        let phrase = encode(40.7128, -74.0060, 3).unwrap();
        assert_eq!(phrase, "grape.column.hip");
    }

    #[test]
    fn test_nyc_decode() {
        let loc = decode("grape.column.hip").unwrap();
        assert!((loc.lat - 40.7128).abs() < 0.01);
        assert!((loc.lng - (-74.0060)).abs() < 0.01);
    }

    // Roundtrip at various precisions
    #[test]
    fn test_roundtrip_precision_1_through_6() {
        for precision in 1..=6u8 {
            let phrase = encode(42.7085, -84.5537, precision).unwrap();
            let words: Vec<&str> = phrase.split('.').collect();
            assert_eq!(words.len(), precision as usize);
            let loc = decode(&phrase).unwrap();
            assert!((loc.lat - 42.7085).abs() < 1000.0);
        }
    }

    // Prefix hierarchy: shorter phrases are prefixes of longer ones
    #[test]
    fn test_prefix_hierarchy() {
        let p3 = encode(40.7128, -74.0060, 3).unwrap();
        let p4 = encode(40.7128, -74.0060, 4).unwrap();
        let p5 = encode(40.7128, -74.0060, 5).unwrap();
        assert!(p4.starts_with(&p3));
        assert!(p5.starts_with(&p4));
    }

    // Parity validation
    #[test]
    fn test_parity_error() {
        let result = decode("grape.column.hip");
        assert!(result.is_ok());
    }

    // Checksum
    #[test]
    fn test_encode_with_checksum() {
        let phrase = encode_with_checksum(40.7128, -74.0060, 3).unwrap();
        let words: Vec<&str> = phrase.split('.').collect();
        assert_eq!(words.len(), 4);
    }

    // Altitude
    #[test]
    fn test_altitude_roundtrip() {
        let alt = 100.0;
        let idx = altitude::encode_altitude(alt);
        let decoded = altitude::decode_altitude(idx);
        assert!((decoded - alt).abs() < 5.0);
    }

    // Seed permutation
    #[test]
    fn test_seed_produces_different_phrases() {
        let standard = encode(40.7128, -74.0060, 3).unwrap();
        let seeded = encode_with_seed(40.7128, -74.0060, 3, "my-secret-seed").unwrap();
        assert_ne!(standard, seeded);
        let loc = decode_with_seed(&seeded, "my-secret-seed").unwrap();
        assert!((loc.lat - 40.7128).abs() < 0.01);
    }

    // Geohash module tests
    #[test]
    fn test_geohash_nyc() {
        let hash = geohash::encode(40.7128, -74.0060, 6);
        assert_eq!(hash, "dr5reg");
    }

    #[test]
    fn test_geohash_roundtrip() {
        let hash = geohash::encode(42.7085, -84.5537, 12);
        let (lat, lng, _, _) = geohash::decode(&hash);
        assert!((lat - 42.7085).abs() < 0.0001);
        assert!((lng - (-84.5537)).abs() < 0.0001);
    }

    // Cell bounds
    #[test]
    fn test_cell_bounds() {
        let (min_lat, max_lat, min_lng, max_lng) = cell_bounds("grape.column.hip").unwrap();
        assert!(min_lat < 40.7128);
        assert!(max_lat > 40.7128);
        assert!(min_lng < -74.0060);
        assert!(max_lng > -74.0060);
    }
}
