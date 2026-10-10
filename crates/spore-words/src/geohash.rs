//! Geohash encode/decode (standard algorithm, no external dependencies).

/// Base32 alphabet used by geohash (note: no 'a', 'i', 'l', 'o').
const BASE32: &[u8; 32] = b"0123456789bcdefghjkmnpqrstuvwxyz";

/// Public base32 alphabet for SayWhere chunk encoding.
pub(crate) const BASE32_CHARS: &[u8; 32] = BASE32;

#[inline]
fn base32_index(c: u8) -> Option<usize> {
    BASE32.iter().position(|&b| b == c)
}

/// Encode latitude/longitude to geohash string of given precision (number of characters).
pub fn encode(lat: f64, lng: f64, precision: usize) -> String {
    let mut lat_range = (-90.0_f64, 90.0_f64);
    let mut lng_range = (-180.0_f64, 180.0_f64);
    let mut hash = String::with_capacity(precision);
    let mut bits = 0_u32;
    let mut bit_count = 0_u32;
    let mut is_lng = true;

    for _ in 0..precision {
        while bit_count < 5 {
            if is_lng {
                let mid = (lng_range.0 + lng_range.1) / 2.0;
                if lng >= mid {
                    bits = (bits << 1) | 1;
                    lng_range.0 = mid;
                } else {
                    bits <<= 1;
                    lng_range.1 = mid;
                }
            } else {
                let mid = (lat_range.0 + lat_range.1) / 2.0;
                if lat >= mid {
                    bits = (bits << 1) | 1;
                    lat_range.0 = mid;
                } else {
                    bits <<= 1;
                    lat_range.1 = mid;
                }
            }
            is_lng = !is_lng;
            bit_count += 1;
        }
        #[allow(clippy::cast_possible_truncation)]
        {
            hash.push(BASE32[bits as usize] as char);
        }
        bits = 0;
        bit_count = 0;
    }

    hash
}

fn decode_ranges(hash: &str) -> ((f64, f64), (f64, f64)) {
    let mut lat_range = (-90.0_f64, 90.0_f64);
    let mut lng_range = (-180.0_f64, 180.0_f64);
    let mut is_lng = true;

    for c in hash.bytes() {
        let idx = base32_index(c).unwrap_or(0);
        for shift in (0..5).rev() {
            let bit = (idx >> shift) & 1;
            if is_lng {
                let mid = (lng_range.0 + lng_range.1) / 2.0;
                if bit == 1 {
                    lng_range.0 = mid;
                } else {
                    lng_range.1 = mid;
                }
            } else {
                let mid = (lat_range.0 + lat_range.1) / 2.0;
                if bit == 1 {
                    lat_range.0 = mid;
                } else {
                    lat_range.1 = mid;
                }
            }
            is_lng = !is_lng;
        }
    }

    (lat_range, lng_range)
}

/// Decode geohash string to `(lat, lng, lat_error, lng_error)`.
pub fn decode(hash: &str) -> (f64, f64, f64, f64) {
    let (lat_range, lng_range) = decode_ranges(hash);
    let lat = (lat_range.0 + lat_range.1) / 2.0;
    let lng = (lng_range.0 + lng_range.1) / 2.0;
    let lat_error = (lat_range.1 - lat_range.0) / 2.0;
    let lng_error = (lng_range.1 - lng_range.0) / 2.0;
    (lat, lng, lat_error, lng_error)
}

/// Decode geohash to bounding box: `(min_lat, max_lat, min_lng, max_lng)`.
pub fn decode_bounds(hash: &str) -> (f64, f64, f64, f64) {
    let (lat_range, lng_range) = decode_ranges(hash);
    (lat_range.0, lat_range.1, lng_range.0, lng_range.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_nyc_six_chars() {
        assert_eq!(encode(40.7128, -74.0060, 6), "dr5reg");
    }
}
