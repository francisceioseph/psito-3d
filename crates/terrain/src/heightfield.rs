//! A square grid of heights.

use std::path::Path;

use crate::TerrainError;
use crate::noise::smoothstep;

/// A square grid of heights from 0 (the floor) to 1 (the highest point).
///
/// Row `y` and column `x` map to the world point
/// `(x / (size-1) - 0.5, y / (size-1) - 0.5) * extent` on `x` and `z`.
#[derive(Debug, Clone, PartialEq)]
pub struct Heightfield {
    size: u32,
    data: Vec<f32>,
}

impl Heightfield {
    /// A flat field at height `level`.
    pub fn flat(size: u32, level: f32) -> Self {
        let size = size.max(2);
        Self { size, data: vec![level; (size * size) as usize] }
    }

    /// A field from `size × size` samples given row by row. Values are
    /// clamped to `[0, 1]`, and anything not finite becomes 0.
    pub fn from_samples(size: u32, data: Vec<f32>) -> Result<Self, TerrainError> {
        if size < 2 || data.len() != (size as usize).pow(2) {
            return Err(TerrainError::BadSize { size, len: data.len() });
        }
        let data = data.into_iter().map(|h| if h.is_finite() { h.clamp(0.0, 1.0) } else { 0.0 }).collect();
        Ok(Self { size, data })
    }

    /// A field filling each sample with `f(u, v)`, where `u` and `v` go from
    /// -1 to 1 across the field. Stops early with [`TerrainError::Cancelled`]
    /// when `cancelled` returns true, which is checked once per row.
    pub fn from_fn(
        size: u32,
        mut cancelled: impl FnMut() -> bool,
        mut f: impl FnMut(f32, f32) -> Result<f32, TerrainError>,
    ) -> Result<Self, TerrainError> {
        let size = size.max(2);
        let step = 2.0 / (size - 1) as f32;
        let mut data = Vec::with_capacity((size * size) as usize);
        for y in 0..size {
            if cancelled() {
                return Err(TerrainError::Cancelled);
            }
            let v = -1.0 + y as f32 * step;
            for x in 0..size {
                data.push(f(-1.0 + x as f32 * step, v)?);
            }
        }
        Self::from_samples(size, data)
    }

    /// Reads a picture as heights from its brightness, resized to
    /// `size × size`. Pictures that are not square are stretched.
    pub fn from_image_file(path: &Path, size: u32) -> Result<Self, TerrainError> {
        let img = image::open(path).map_err(|e| TerrainError::Image(e.to_string()))?;
        Ok(Self::from_image(&img, size))
    }

    /// Like [`Self::from_image_file`], for a picture already in memory.
    pub fn from_image(img: &image::DynamicImage, size: u32) -> Self {
        let size = size.max(2);
        let grey = img.resize_exact(size, size, image::imageops::FilterType::Triangle).to_luma32f();
        Self { size, data: grey.into_raw().into_iter().map(|h| h.clamp(0.0, 1.0)).collect() }
    }

    /// Samples along each side.
    pub fn size(&self) -> u32 {
        self.size
    }

    /// The samples, row by row.
    pub fn samples(&self) -> &[f32] {
        &self.data
    }

    /// The height at column `x` and row `y`.
    pub fn get(&self, x: u32, y: u32) -> f32 {
        self.data[(y.min(self.size - 1) * self.size + x.min(self.size - 1)) as usize]
    }

    /// Sets the height at column `x` and row `y`, clamped to `[0, 1]`.
    /// Coordinates outside the field are ignored.
    pub fn set(&mut self, x: u32, y: u32, h: f32) {
        if x < self.size && y < self.size {
            self.data[(y * self.size + x) as usize] = if h.is_finite() { h.clamp(0.0, 1.0) } else { 0.0 };
        }
    }

    /// The height at `(u, v)` in `[-1, 1]`, interpolated between samples.
    pub fn sample(&self, u: f32, v: f32) -> f32 {
        let n = (self.size - 1) as f32;
        let fx = ((u * 0.5 + 0.5) * n).clamp(0.0, n);
        let fy = ((v * 0.5 + 0.5) * n).clamp(0.0, n);
        let (x0, y0) = (fx.floor() as u32, fy.floor() as u32);
        let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
        let a = self.get(x0, y0) + (self.get(x0 + 1, y0) - self.get(x0, y0)) * tx;
        let b = self.get(x0, y0 + 1) + (self.get(x0 + 1, y0 + 1) - self.get(x0, y0 + 1)) * tx;
        a + (b - a) * ty
    }

    /// The lowest and the highest sample.
    pub fn range(&self) -> (f32, f32) {
        self.data.iter().fold((f32::MAX, f32::MIN), |(lo, hi), &h| (lo.min(h), hi.max(h)))
    }

    /// Slopes the border down to 0 over `width` (0 to 0.5) of the side, so
    /// the terrain ends in the sea instead of in a wall.
    pub fn apply_edge_falloff(&mut self, width: f32) {
        let width = width.clamp(0.0, 0.5);
        if width <= 0.0 {
            return;
        }
        let n = (self.size - 1) as f32;
        for y in 0..self.size {
            for x in 0..self.size {
                // Distance to the nearest edge, from 0 at the edge to 0.5 in the middle.
                let d = (x as f32 / n).min(1.0 - x as f32 / n).min(y as f32 / n).min(1.0 - y as f32 / n);
                self.data[(y * self.size + x) as usize] *= smoothstep(0.0, width, d);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_samples_checks_and_clamps() {
        assert!(Heightfield::from_samples(3, vec![0.0; 8]).is_err());
        let h = Heightfield::from_samples(2, vec![-1.0, 2.0, f32::NAN, 0.5]).unwrap();
        assert_eq!(h.samples(), &[0.0, 1.0, 0.0, 0.5]);
    }

    #[test]
    fn sample_interpolates() {
        let h = Heightfield::from_samples(2, vec![0.0, 1.0, 0.0, 1.0]).unwrap();
        assert!((h.sample(0.0, 0.0) - 0.5).abs() < 1e-6);
        assert_eq!(h.sample(-1.0, -1.0), 0.0);
        assert_eq!(h.sample(5.0, 5.0), 1.0, "outside clamps to the edge");
    }

    #[test]
    fn falloff_zeroes_the_border_and_keeps_the_middle() {
        let mut h = Heightfield::flat(65, 1.0);
        h.apply_edge_falloff(0.2);
        assert_eq!(h.get(0, 32), 0.0);
        assert_eq!(h.get(64, 64), 0.0);
        assert_eq!(h.get(32, 32), 1.0);
    }

    #[test]
    fn from_fn_covers_minus_one_to_one() {
        let h = Heightfield::from_fn(3, || false, |u, v| Ok((u + v + 2.0) / 4.0)).unwrap();
        assert_eq!(h.get(0, 0), 0.0);
        assert_eq!(h.get(2, 2), 1.0);
        assert!((h.get(1, 1) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn from_fn_can_be_cancelled() {
        let r = Heightfield::from_fn(8, || true, |_, _| Ok(0.0));
        assert!(matches!(r, Err(TerrainError::Cancelled)));
    }

    #[test]
    fn image_brightness_becomes_height() {
        let mut img = image::GrayImage::new(4, 4);
        for (x, _, p) in img.enumerate_pixels_mut() {
            p.0 = [if x < 2 { 0 } else { 255 }];
        }
        let h = Heightfield::from_image(&image::DynamicImage::ImageLuma8(img), 4);
        assert!(h.get(0, 0) < 0.05);
        assert!(h.get(3, 3) > 0.95);
    }
}
