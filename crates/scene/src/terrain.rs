//! The recipe of the terrain.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::ParamValues;

/// Where the heights come from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TerrainSource {
    /// A generator, built in or from a plugin, with its parameters.
    Generator {
        /// The generator id, such as `psito.hills` or a plugin's own id.
        id: String,
        /// The values of its parameters.
        #[serde(default)]
        params: ParamValues,
    },
    /// A greyscale picture: white is high and black is low. This is how a
    /// doodle from Psito Plume becomes a mountain range.
    Image {
        /// The picture, relative to the scene file when it was saved.
        path: PathBuf,
    },
}

/// What a brush does to the ground.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BrushTool {
    /// Pushes the ground up.
    Raise,
    /// Digs it down.
    Lower,
    /// Evens out bumps.
    Smooth,
    /// Levels the ground to the height where the stroke began.
    Flatten,
}

impl BrushTool {
    /// Every tool, in the order of the panel.
    pub const ALL: [BrushTool; 4] = [BrushTool::Raise, BrushTool::Lower, BrushTool::Smooth, BrushTool::Flatten];
}

/// One stroke of a brush over the terrain: drawn once the generator has
/// made the ground, so the terrain stays a recipe that rebuilds the same
/// world, and a stroke is one undo step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrushStroke {
    /// What the brush does.
    pub tool: BrushTool,
    /// The radius, in world units.
    pub radius: f32,
    /// How strong each touch is, from 0 to 1.
    pub strength: f32,
    /// Where the brush passed, as world `x` and `z`.
    pub points: Vec<[f32; 2]>,
}

impl BrushStroke {
    /// The limits of the radius.
    pub const RADIUS_RANGE: (f32, f32) = (0.5, 40.0);
}

/// The terrain: a square heightfield in the middle of the world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TerrainSettings {
    /// Where the heights come from.
    pub source: TerrainSource,
    /// The seed handed to the generator.
    pub seed: u64,
    /// Samples along each side of the heightfield.
    pub resolution: u32,
    /// The length of each side, in world units.
    pub extent: f32,
    /// The height of the highest possible point, in world units.
    pub height: f32,
    /// The width of the border that slopes down to the floor, from 0 to 0.5
    /// of the side, so the terrain ends in the sea and not in a wall.
    pub edge_falloff: f32,
    /// Brush strokes painted over the generated ground, in order.
    pub strokes: Vec<BrushStroke>,
}

impl TerrainSettings {
    /// The smallest and the largest resolution accepted.
    pub const RESOLUTION_RANGE: (u32, u32) = (16, 2048);

    /// The most brush points a terrain keeps, over all its strokes.
    pub const MAX_BRUSH_POINTS: usize = 50_000;

    /// These settings without the brush strokes: what the generator makes.
    pub fn without_strokes(&self) -> Self {
        Self { strokes: Vec::new(), ..self.clone() }
    }

    /// Drops broken strokes and points, clamps the brushes, and keeps at
    /// most [`Self::MAX_BRUSH_POINTS`] points.
    pub fn sanitize_strokes(&mut self) {
        let mut budget = Self::MAX_BRUSH_POINTS;
        self.strokes.retain_mut(|s| {
            s.points.retain(|p| p[0].is_finite() && p[1].is_finite());
            s.points.truncate(budget);
            budget -= s.points.len();
            s.radius = if s.radius.is_finite() {
                s.radius.clamp(BrushStroke::RADIUS_RANGE.0, BrushStroke::RADIUS_RANGE.1)
            } else {
                BrushStroke::RADIUS_RANGE.0
            };
            s.strength = if s.strength.is_finite() { s.strength.clamp(0.0, 1.0) } else { 0.0 };
            !s.points.is_empty()
        });
    }

    /// The resolution clamped to [`Self::RESOLUTION_RANGE`].
    pub fn clamped_resolution(&self) -> u32 {
        self.resolution.clamp(Self::RESOLUTION_RANGE.0, Self::RESOLUTION_RANGE.1)
    }
}

impl Default for TerrainSettings {
    fn default() -> Self {
        Self {
            source: TerrainSource::Generator { id: "psito.hills".to_owned(), params: ParamValues::new() },
            seed: 1,
            resolution: 512,
            extent: 100.0,
            height: 24.0,
            edge_falloff: 0.18,
            strokes: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stroke(points: Vec<[f32; 2]>) -> BrushStroke {
        BrushStroke { tool: BrushTool::Raise, radius: 5.0, strength: 0.5, points }
    }

    #[test]
    fn sanitize_drops_broken_strokes_and_limits_points() {
        let mut t = TerrainSettings {
            strokes: vec![
                stroke(vec![]),
                stroke(vec![[f32::NAN, 0.0]]),
                BrushStroke { radius: 1e9, strength: -2.0, ..stroke(vec![[1.0, 2.0]]) },
                stroke(vec![[0.0, 0.0]; TerrainSettings::MAX_BRUSH_POINTS + 10]),
            ],
            ..Default::default()
        };
        t.sanitize_strokes();
        assert_eq!(t.strokes.len(), 2);
        assert_eq!((t.strokes[0].radius, t.strokes[0].strength), (BrushStroke::RADIUS_RANGE.1, 0.0));
        let total: usize = t.strokes.iter().map(|s| s.points.len()).sum();
        assert_eq!(total, TerrainSettings::MAX_BRUSH_POINTS);
    }

    #[test]
    fn without_strokes_keeps_the_rest() {
        let mut t = TerrainSettings { seed: 9, ..Default::default() };
        t.strokes.push(stroke(vec![[0.0, 0.0]]));
        let base = t.without_strokes();
        assert!(base.strokes.is_empty());
        assert_eq!(base.seed, 9);
    }
}
