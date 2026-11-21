//! The scene model of Psito 3D.
//!
//! A [`Scene`] is plain data: it says *what* the world looks like (the
//! terrain recipe, the sky, the sea, the objects and the camera) and never
//! *how* it is drawn. The renderer, the terrain generators and the editor all
//! read it, and only the editor changes it, through the [`History`], so every
//! change can be undone.
//!
//! The world uses a right-handed system with `y` up. The terrain covers the
//! square `[-extent/2, extent/2]` on `x` and `z`, and its heights go from `0`
//! to [`TerrainSettings::height`].

mod camera;
mod code;
mod history;
mod object;
mod params;
mod rng;
mod scene;
mod sky;
mod terrain;

pub use camera::Camera;
pub use code::CODE_PREFIX;
pub use history::History;
pub use object::{Combine, Material, ObjectId, SceneObject, Shape, Transform};
pub use params::{
    ChoiceOption, LocalizedText, ParamKind, ParamSpec, ParamValue, ParamValues, resolve as resolve_params,
};
pub use rng::SplitMix64;
pub use scene::{FORMAT_VERSION, MAX_OBJECTS, Scene, SceneError};
pub use sky::{Filter, Fog, Sea, Sky, Sun};
pub use terrain::{BrushStroke, BrushTool, TerrainSettings, TerrainSource};
