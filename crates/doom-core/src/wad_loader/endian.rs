// SPDX-License-Identifier: AGPL-3.0-or-later
//! Little-endian primitive reads for WAD lump records.

/// Read a little-endian `i32` from `buf` starting at `start` (16-byte directory entries, 12-byte header).
#[inline]
pub(crate) fn i32_le_at(buf: &[u8], start: usize) -> i32 {
    i32::from_le_bytes([buf[start], buf[start + 1], buf[start + 2], buf[start + 3]])
}

#[inline]
pub(crate) fn i16_le(slice: &[u8]) -> i16 {
    i16::from_le_bytes([slice[0], slice[1]])
}

#[inline]
pub(crate) fn u16_le(slice: &[u8]) -> u16 {
    u16::from_le_bytes([slice[0], slice[1]])
}
