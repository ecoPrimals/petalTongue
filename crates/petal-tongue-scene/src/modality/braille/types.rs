// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};

/// A single Braille cell (3x2 dot matrix, Unicode Braille block U+2800..U+28FF).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrailleCell {
    pub dots: u8,
}

impl BrailleCell {
    pub const BLANK: Self = Self { dots: 0 };

    #[expect(clippy::cast_lossless, reason = "u8 to u32 is always lossless")]
    #[must_use]
    pub fn to_char(self) -> char {
        char::from_u32(0x2800 + self.dots as u32).unwrap_or('\u{2800}')
    }
}
