//! The generator trait and the job that runs one.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use psito3d_scene::{LocalizedText, ParamSpec, ParamValues, TerrainSettings, TerrainSource, resolve_params};
use serde::Deserialize;

use crate::Heightfield;

/// What can go wrong making a terrain.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TerrainError {
    /// A newer request replaced this one.
    #[error("cancelled")]
    Cancelled,
    /// No generator has this id (a plugin that was removed, say).
    #[error("no terrain generator called {0:?}")]
    UnknownGenerator(String),
    /// The samples do not fill a square.
    #[error("{len} samples do not make a {size}×{size} field")]
    BadSize {
        /// The side asked for.
        size: u32,
        /// The samples there were.
        len: usize,
    },
    /// The picture could not be read.
    #[error("could not read the picture: {0}")]
    Image(String),
    /// A plugin failed; the message comes from Lua.
    #[error("the plugin failed: {0}")]
    Plugin(String),
    /// A generator description is not valid.
    #[error("invalid generator description: {0}")]
    BadDescription(String),
}

/// Lets a running generation be stopped from another thread, so dragging a
/// slider does not queue up a pile of stale terrains.
#[derive(Debug, Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    /// A token that is not cancelled.
    pub fn new() -> Self {
        Self::default()
    }

    /// Asks the generation to stop.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    /// Whether [`Self::cancel`] was called.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// What a generator is asked to make.
#[derive(Debug, Clone)]
pub struct GenerateRequest {
    /// The seed.
    pub seed: u64,
    /// Samples along each side.
    pub resolution: u32,
    /// The parameter values, already resolved against the generator's specs.
    pub params: ParamValues,
}

/// The description of a generator: what the editor shows before running it.
/// Built-in generators and plugins write it in the same TOML format.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GeneratorInfo {
    /// The id stored in scenes, such as `psito.hills`.
    pub id: String,
    /// The name in the generator menu.
    pub name: LocalizedText,
    /// One sentence on what it makes.
    #[serde(default)]
    pub description: LocalizedText,
    /// The parameters, in the order shown.
    #[serde(default)]
    pub params: Vec<ParamSpec>,
}

impl GeneratorInfo {
    /// Reads a description from TOML and checks it.
    pub fn from_toml(text: &str) -> Result<Self, TerrainError> {
        let info: Self = toml::from_str(text).map_err(|e| TerrainError::BadDescription(e.to_string()))?;
        info.validate()?;
        Ok(info)
    }

    /// Checks the id and every parameter, and that no two share an id.
    pub fn validate(&self) -> Result<(), TerrainError> {
        let bad = |m: String| Err(TerrainError::BadDescription(m));
        if self.id.is_empty() || !self.id.chars().all(|c| c.is_ascii_alphanumeric() || "._-".contains(c)) {
            return bad(format!("generator id {:?} must be letters, digits, '.', '_' or '-'", self.id));
        }
        let mut seen = std::collections::BTreeSet::new();
        for p in &self.params {
            p.validate().map_err(TerrainError::BadDescription)?;
            if !seen.insert(&p.id) {
                return bad(format!("parameter {} appears twice", p.id));
            }
        }
        Ok(())
    }
}

/// A way of making a terrain (the Strategy pattern).
///
/// Implementations must be deterministic: the same request gives the same
/// heights, so a scene file and its seed always rebuild the same world.
pub trait TerrainGenerator: Send + Sync {
    /// The description shown in the editor.
    fn info(&self) -> &GeneratorInfo;

    /// Makes the heights, from 0 to 1. Long work should check `cancel`
    /// often and return [`TerrainError::Cancelled`] when it is set.
    fn generate(&self, request: &GenerateRequest, cancel: &CancelToken) -> Result<Heightfield, TerrainError>;
}

/// Builds the heightfield of `settings`: runs the generator, or reads the
/// picture, slopes the border and paints the brush strokes.
///
/// `find` looks a generator up by id, and `base_dir` resolves the relative
/// picture paths of a scene file.
pub fn generate_terrain<'a>(
    settings: &TerrainSettings,
    find: impl Fn(&str) -> Option<&'a dyn TerrainGenerator>,
    base_dir: Option<&std::path::Path>,
    cancel: &CancelToken,
) -> Result<Heightfield, TerrainError> {
    let mut field = generate_base(settings, find, base_dir, cancel)?;
    crate::apply_strokes(&mut field, &settings.strokes, settings.extent);
    Ok(field)
}

/// Like [`generate_terrain`], without the brush strokes: the ground the
/// generator makes. The editor keeps it, so painting only redraws strokes.
pub fn generate_base<'a>(
    settings: &TerrainSettings,
    find: impl Fn(&str) -> Option<&'a dyn TerrainGenerator>,
    base_dir: Option<&std::path::Path>,
    cancel: &CancelToken,
) -> Result<Heightfield, TerrainError> {
    let resolution = settings.clamped_resolution();
    let mut field = match &settings.source {
        TerrainSource::Generator { id, params } => {
            let generator = find(id).ok_or_else(|| TerrainError::UnknownGenerator(id.clone()))?;
            let request = GenerateRequest {
                seed: settings.seed,
                resolution,
                params: resolve_params(&generator.info().params, params),
            };
            let field = generator.generate(&request, cancel)?;
            if field.size() != resolution {
                return Err(TerrainError::BadSize { size: resolution, len: field.samples().len() });
            }
            field
        }
        TerrainSource::Image { path } => {
            let path = match base_dir {
                Some(dir) if path.is_relative() => dir.join(path),
                _ => path.clone(),
            };
            Heightfield::from_image_file(&path, resolution)?
        }
    };
    if cancel.is_cancelled() {
        return Err(TerrainError::Cancelled);
    }
    field.apply_edge_falloff(settings.edge_falloff);
    Ok(field)
}
