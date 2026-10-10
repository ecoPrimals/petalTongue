//! Altitude quantization (IETF draft-saywhere-geocoding section 10).

const ALTITUDE_MIN: f64 = -500.0;
const ALTITUDE_RANGE: f64 = 10_000.0;
const ALTITUDE_LEVELS: f64 = 2047.0;

/// Quantize altitude in meters to an 11-bit word index (0-2047).
///
/// Range: -500m to 9500m (covers Dead Sea to Everest with margin).
pub fn encode_altitude(meters: f64) -> u16 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    {
        ((meters + ALTITUDE_MIN.abs()) / ALTITUDE_RANGE * ALTITUDE_LEVELS)
            .round()
            .clamp(0.0, ALTITUDE_LEVELS) as u16
    }
}

/// Decode altitude word index back to meters.
pub fn decode_altitude(index: u16) -> f64 {
    (f64::from(index) / ALTITUDE_LEVELS) * ALTITUDE_RANGE - ALTITUDE_MIN.abs()
}
