//! A small deterministic random number generator.

/// SplitMix64, the generator from Steele, Lea and Flood (2014).
///
/// It is tiny, fast and good enough for picking parameters: the same seed
/// always gives the same sequence on every platform, which is what makes a
/// "Surprise me" world reproducible from its seed.
#[derive(Debug, Clone)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Starts the sequence from `seed`.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// The next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A float in `[0, 1)`.
    pub fn next_f32(&mut self) -> f32 {
        // The top 24 bits fill the mantissa exactly.
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// A float in `[min, max)`.
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }

    /// An index in `[0, len)`. `len` must not be zero.
    pub fn index(&mut self, len: usize) -> usize {
        assert!(len > 0, "cannot pick from an empty range");
        (self.next_u64() % len as u64) as usize
    }

    /// `true` with probability `p`.
    pub fn chance(&mut self, p: f32) -> bool {
        self.next_f32() < p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn known_first_value() {
        // The reference output of SplitMix64 for the seed 0.
        assert_eq!(SplitMix64::new(0).next_u64(), 0xE220_A839_7B1D_CDAF);
    }

    #[test]
    fn floats_stay_in_range() {
        let mut r = SplitMix64::new(7);
        for _ in 0..10_000 {
            let f = r.next_f32();
            assert!((0.0..1.0).contains(&f));
            let g = r.range(-3.0, 5.0);
            assert!((-3.0..5.0).contains(&g));
            assert!(r.index(3) < 3);
        }
    }
}
