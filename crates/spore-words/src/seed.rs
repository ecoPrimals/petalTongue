//! Custom wordlist permutation via BLAKE3-seeded Fisher-Yates shuffle.

use blake3::Hasher;

use crate::wordlist::BIP39_EN;

/// PRNG backed by BLAKE3 keyed hash extension of the seed digest.
struct SeedRng {
    key: [u8; 32],
    buffer: [u8; 32],
    offset: usize,
    block: u64,
}

impl SeedRng {
    fn new(seed: &str) -> Self {
        let key = *blake3::hash(seed.as_bytes()).as_bytes();
        Self {
            key,
            buffer: [0; 32],
            offset: 32,
            block: 0,
        }
    }

    fn refill(&mut self) {
        let mut hasher = Hasher::new_keyed(&self.key);
        hasher.update(&self.block.to_le_bytes());
        self.buffer = *hasher.finalize().as_bytes();
        self.offset = 0;
        self.block += 1;
    }

    fn next_byte(&mut self) -> u8 {
        if self.offset >= 32 {
            self.refill();
        }
        let b = self.buffer[self.offset];
        self.offset += 1;
        b
    }

    fn next_u32(&mut self) -> u32 {
        u32::from(self.next_byte())
            | (u32::from(self.next_byte()) << 8)
            | (u32::from(self.next_byte()) << 16)
            | (u32::from(self.next_byte()) << 24)
    }

    fn next_bounded(&mut self, upper: usize) -> usize {
        debug_assert!(upper > 0);
        // Fisher-Yates passes `i + 1` where `i <= 2047`, so this always fits in u32.
        #[allow(clippy::cast_possible_truncation)]
        let upper_u32 = upper as u32;
        let limit = u32::MAX - u32::MAX % upper_u32;
        loop {
            let sample = self.next_u32();
            if sample < limit {
                return sample as usize % upper;
            }
        }
    }
}

/// Generate a permuted wordlist from a seed string.
///
/// Uses Fisher-Yates shuffle with BLAKE3-derived random bytes.
pub fn permute_wordlist(seed: &str) -> [&'static str; 2048] {
    let mut words = BIP39_EN;
    let mut rng = SeedRng::new(seed);

    for i in (1..2048).rev() {
        let j = rng.next_bounded(i + 1);
        words.swap(i, j);
    }

    words
}
