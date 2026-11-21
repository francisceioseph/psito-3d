//! The scene and its file format.

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::{Camera, Filter, Fog, Material, ObjectId, SceneObject, Sea, Shape, Sky, TerrainSettings, Transform};

/// The version written in new scene files. A file with a newer version is
/// refused instead of being read wrong.
pub const FORMAT_VERSION: u32 = 1;

/// The most objects a scene can have; the renderer sizes its buffer for it.
pub const MAX_OBJECTS: usize = 64;

/// What can go wrong reading a scene.
#[derive(Debug, thiserror::Error)]
pub enum SceneError {
    /// The file is not a scene.
    #[error("not a Psito 3D scene: {0}")]
    Parse(#[from] serde_json::Error),
    /// The file comes from a newer version of the program.
    #[error("the scene was saved by a newer Psito 3D (format {found}, this one reads up to {FORMAT_VERSION})")]
    TooNew {
        /// The version in the file.
        found: u32,
    },
}

/// Everything that makes up a world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Scene {
    /// The file format version.
    pub format: u32,
    /// The terrain recipe.
    pub terrain: TerrainSettings,
    /// The sky and the sun.
    pub sky: Sky,
    /// The fog and the mist.
    pub fog: Fog,
    /// The sea.
    pub sea: Sea,
    /// The objects, in the order they are combined.
    pub objects: Vec<SceneObject>,
    /// The camera.
    pub camera: Camera,
    /// The finishing filter.
    pub filter: Filter,
    /// The id the next new object gets.
    pub next_object_id: u64,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            format: FORMAT_VERSION,
            terrain: TerrainSettings::default(),
            sky: Sky::default(),
            fog: Fog::default(),
            sea: Sea::default(),
            objects: Vec::new(),
            camera: Camera::default(),
            filter: Filter::None,
            next_object_id: 1,
        }
    }
}

impl Scene {
    /// The starting scene: an island, the sea, a late afternoon sun and a
    /// chrome ball floating over the water.
    pub fn starter() -> Self {
        let mut scene = Self::default();
        scene.terrain.source =
            crate::TerrainSource::Generator { id: "psito.island".to_owned(), params: Default::default() };
        let id = scene.add_object(Shape::Sphere, Material::chrome()).expect("an empty scene has room");
        if let Some(ball) = scene.object_mut(id) {
            ball.transform =
                Transform { position: Vec3::new(14.0, 9.0, 18.0), scale: Vec3::splat(7.0), ..Default::default() };
        }
        scene
    }

    /// Adds an object at the camera target and returns its id, or `None`
    /// when the scene already has [`MAX_OBJECTS`].
    pub fn add_object(&mut self, shape: Shape, material: Material) -> Option<ObjectId> {
        if self.objects.len() >= MAX_OBJECTS {
            return None;
        }
        let id = ObjectId(self.next_object_id);
        self.next_object_id += 1;
        self.objects.push(SceneObject {
            id,
            name: format!("{shape:?} {}", id.0),
            shape,
            transform: Transform { position: self.camera.target, ..Default::default() },
            material,
            combine: Default::default(),
            blend: 0.0,
            visible: true,
        });
        Some(id)
    }

    /// The object with `id`.
    pub fn object(&self, id: ObjectId) -> Option<&SceneObject> {
        self.objects.iter().find(|o| o.id == id)
    }

    /// The object with `id`, to change it.
    pub fn object_mut(&mut self, id: ObjectId) -> Option<&mut SceneObject> {
        self.objects.iter_mut().find(|o| o.id == id)
    }

    /// Removes the object with `id`; returns whether there was one.
    pub fn remove_object(&mut self, id: ObjectId) -> bool {
        let before = self.objects.len();
        self.objects.retain(|o| o.id != id);
        self.objects.len() != before
    }

    /// The scene as pretty JSON, the `.psito3d` format.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("a scene is always serialisable")
    }

    /// Reads a `.psito3d` file. Missing fields take their defaults and
    /// values out of range are clamped, so a file edited by hand still opens.
    pub fn from_json(text: &str) -> Result<Self, SceneError> {
        let mut scene: Scene = serde_json::from_str(text)?;
        if scene.format > FORMAT_VERSION {
            return Err(SceneError::TooNew { found: scene.format });
        }
        scene.format = FORMAT_VERSION;
        scene.sanitize();
        Ok(scene)
    }

    /// Puts every value in its valid range.
    pub fn sanitize(&mut self) {
        let unit = |v: f32| if v.is_finite() { v.clamp(0.0, 1.0) } else { 0.0 };
        self.objects.truncate(MAX_OBJECTS);
        for o in &mut self.objects {
            o.material = o.material.sanitized();
            o.blend = if o.blend.is_finite() { o.blend.clamp(0.0, 20.0) } else { 0.0 };
        }
        let max_id = self.objects.iter().map(|o| o.id.0).max().unwrap_or(0);
        self.next_object_id = self.next_object_id.max(max_id + 1);
        self.sky.cloud_cover = unit(self.sky.cloud_cover);
        self.sky.haze = unit(self.sky.haze);
        self.fog.density = unit(self.fog.density);
        self.fog.mist = unit(self.fog.mist);
        self.sea.level = unit(self.sea.level);
        self.sea.ripple = unit(self.sea.ripple);
        self.sea.color = self.sea.color.map(unit);
        self.terrain.resolution = self.terrain.clamped_resolution();
        self.terrain.edge_falloff = self.terrain.edge_falloff.clamp(0.0, 0.5);
        self.camera.pitch = self.camera.pitch.clamp(Camera::PITCH_RANGE.0, Camera::PITCH_RANGE.1);
        self.camera.distance = self.camera.distance.clamp(Camera::DISTANCE_RANGE.0, Camera::DISTANCE_RANGE.1);
        self.camera.fov = self.camera.fov.clamp(10.0, 120.0);
    }
}
