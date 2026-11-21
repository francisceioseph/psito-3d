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
}

impl TerrainSettings {
    /// The smallest and the largest resolution accepted.
    pub const RESOLUTION_RANGE: (u32, u32) = (16, 2048);

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
        }
    }
}
