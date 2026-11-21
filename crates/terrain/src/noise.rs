//! Seeded 2D gradient noise and the fractal sums built on it.
//!
//! All functions are pure: the same input and seed give the same output on
//! every platform, which keeps a world reproducible from its seed.

/// Settings of a fractal sum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fractal {
    /// How many layers of noise are added.
    pub octaves: u32,
    /// How much finer each layer is than the one before.
    pub lacunarity: f32,
    /// How much weaker each layer is than the one before.
    pub gain: f32,
}

impl Default for Fractal {
    fn default() -> Self {
        Self { octaves: 6, lacunarity: 2.0, gain: 0.5 }
    }
}

impl Fractal {
    /// The most octaves allowed; beyond this the layers are finer than a sample.
    pub const MAX_OCTAVES: u32 = 16;
}

fn hash(x: i32, y: i32, seed: u32) -> u32 {
    // A small integer hash (after the one by Chris Wellons) over the lattice point.
    let mut h =
        (x as u32).wrapping_mul(0x8DA6_B343) ^ (y as u32).wrapping_mul(0xD816_3841) ^ seed.wrapping_mul(0xCB1A_B31F);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846C_A68B);
    h ^ (h >> 16)
}

fn gradient(x: i32, y: i32, seed: u32) -> (f32, f32) {
    let angle = hash(x, y, seed) as f32 * (std::f32::consts::TAU / u32::MAX as f32);
    (angle.cos(), angle.sin())
}

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Gradient (Perlin-style) noise at `(x, y)`, roughly in `[-1, 1]`, with one
/// feature per unit.
pub fn perlin(x: f32, y: f32, seed: u32) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (ix, iy) = (x0 as i32, y0 as i32);
    let (fx, fy) = (x - x0, y - y0);
    let dot = |gx: i32, gy: i32, dx: f32, dy: f32| {
        let (g0, g1) = gradient(gx, gy, seed);
        g0 * dx + g1 * dy
    };
    let n00 = dot(ix, iy, fx, fy);
    let n10 = dot(ix + 1, iy, fx - 1.0, fy);
    let n01 = dot(ix, iy + 1, fx, fy - 1.0);
    let n11 = dot(ix + 1, iy + 1, fx - 1.0, fy - 1.0);
    let (u, v) = (fade(fx), fade(fy));
    let a = n00 + u * (n10 - n00);
    let b = n01 + u * (n11 - n01);
    // √2 scales the gradient noise to about [-1, 1].
    (a + v * (b - a)) * std::f32::consts::SQRT_2
}

fn octave_seed(seed: u32, octave: u32) -> u32 {
    seed.wrapping_add(octave.wrapping_mul(0x9E37_79B9))
}

/// Fractal Brownian motion: layers of [`perlin`] noise, normalised to about
/// `[-1, 1]`. Rolling hills.
pub fn fbm(x: f32, y: f32, seed: u32, f: Fractal) -> f32 {
    let (mut sum, mut amp, mut freq, mut norm) = (0.0, 1.0, 1.0, 0.0);
    for o in 0..f.octaves.clamp(1, Fractal::MAX_OCTAVES) {
        sum += amp * perlin(x * freq, y * freq, octave_seed(seed, o));
        norm += amp;
        amp *= f.gain;
        freq *= f.lacunarity;
    }
    sum / norm
}

/// Ridged multifractal noise in `[0, 1]`: sharp crests where the noise
/// crosses zero, and each layer stronger on the crests than in the valleys.
/// Mountain ranges.
pub fn ridged(x: f32, y: f32, seed: u32, f: Fractal) -> f32 {
    let (mut sum, mut amp, mut freq, mut norm, mut weight) = (0.0, 1.0, 1.0, 0.0, 1.0);
    for o in 0..f.octaves.clamp(1, Fractal::MAX_OCTAVES) {
        let n = 1.0 - perlin(x * freq, y * freq, octave_seed(seed, o)).abs();
        let n = n * n * weight;
        weight = (n * 2.0).clamp(0.0, 1.0);
        sum += amp * n;
        norm += amp;
        amp *= f.gain;
        freq *= f.lacunarity;
    }
    (sum / norm).clamp(0.0, 1.0)
}

/// The smooth step from 0 at `edge0` to 1 at `edge1`.
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        assert_eq!(perlin(1.3, 2.7, 5), perlin(1.3, 2.7, 5));
        assert_ne!(perlin(1.3, 2.7, 5), perlin(1.3, 2.7, 6));
    }

    #[test]
    fn zero_on_the_lattice() {
        // Gradient noise is zero at the lattice points by construction.
        assert_eq!(perlin(3.0, -4.0, 1), 0.0);
    }

    #[test]
    fn ranges() {
        let f = Fractal::default();
        for i in 0..2000 {
            let (x, y) = (i as f32 * 0.137, i as f32 * 0.071);
            assert!(perlin(x, y, 9).abs() <= 1.05);
            assert!(fbm(x, y, 9, f).abs() <= 1.05);
            assert!((0.0..=1.0).contains(&ridged(x, y, 9, f)));
        }
    }

    #[test]
    fn continuous() {
        let a = fbm(0.5, 0.5, 3, Fractal::default());
        let b = fbm(0.5001, 0.5, 3, Fractal::default());
        assert!((a - b).abs() < 0.01);
    }

    #[test]
    fn octaves_are_clamped() {
        let huge = Fractal { octaves: 10_000, ..Default::default() };
        // Finishes quickly and stays in range.
        assert!(fbm(0.3, 0.3, 1, huge).abs() <= 1.05);
    }
}
