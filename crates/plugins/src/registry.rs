//! Finding and loading the installed plugins.

use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use crate::{Limits, LuaTerrainGenerator, PluginManifest};

/// A plugin that loaded.
#[derive(Debug, Clone)]
pub struct LoadedPlugin {
    /// Its manifest.
    pub manifest: PluginManifest,
    /// Its folder.
    pub dir: PathBuf,
    /// Its terrain generator, if it has one.
    pub generator: Option<Arc<LuaTerrainGenerator>>,
    /// Its theme pack file, if it has one, already checked to be inside the folder.
    pub theme: Option<PathBuf>,
}

/// A plugin that did not load, and why. The editor lists these so the
/// author can fix them.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("{}: {message}", dir.display())]
pub struct PluginError {
    /// The plugin folder.
    pub dir: PathBuf,
    /// What is wrong.
    pub message: String,
}

/// The installed plugins.
#[derive(Debug, Clone, Default)]
pub struct PluginRegistry {
    /// The plugins that loaded, in the order found.
    pub plugins: Vec<LoadedPlugin>,
    /// The ones that did not.
    pub errors: Vec<PluginError>,
}

/// `relative` inside `dir`, refusing anything that points outside it:
/// absolute paths, `..`, and links that lead out.
fn inside(dir: &Path, relative: &str) -> Result<PathBuf, String> {
    let rel = Path::new(relative);
    if rel.components().any(|c| !matches!(c, Component::Normal(_) | Component::CurDir)) {
        return Err(format!("{relative:?} must be a path inside the plugin folder"));
    }
    let path = dir.join(rel);
    let real = path.canonicalize().map_err(|e| format!("{relative}: {e}"))?;
    let real_dir = dir.canonicalize().map_err(|e| e.to_string())?;
    if !real.starts_with(&real_dir) {
        return Err(format!("{relative:?} leads outside the plugin folder"));
    }
    Ok(real)
}

impl PluginRegistry {
    /// Looks for plugins in each of `roots`, in order: every folder in a
    /// root with a `plugin.toml` is a plugin. When two plugins share an id,
    /// the one found first wins, so put the person's own folder first.
    pub fn scan(roots: &[PathBuf], limits: Limits) -> Self {
        let mut registry = Self::default();
        for root in roots {
            let Ok(entries) = std::fs::read_dir(root) else { continue };
            let mut dirs: Vec<PathBuf> =
                entries.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.join("plugin.toml").is_file()).collect();
            dirs.sort();
            for dir in dirs {
                match Self::load(&dir, limits) {
                    Ok(plugin) if registry.get(&plugin.manifest.id).is_some() => registry.errors.push(PluginError {
                        dir,
                        message: format!("another plugin already uses the id {}", plugin.manifest.id),
                    }),
                    Ok(plugin) => registry.plugins.push(plugin),
                    Err(message) => registry.errors.push(PluginError { dir, message }),
                }
            }
        }
        registry
    }

    /// Loads the plugin in `dir`.
    pub fn load(dir: &Path, limits: Limits) -> Result<LoadedPlugin, String> {
        let text = std::fs::read_to_string(dir.join("plugin.toml")).map_err(|e| e.to_string())?;
        let manifest = PluginManifest::from_toml(&text)?;
        let generator = match (&manifest.terrain, manifest.generator_info()) {
            (Some(section), Some(info)) => {
                let script = inside(dir, &section.script)?;
                let source = std::fs::read_to_string(&script).map_err(|e| format!("{}: {e}", section.script))?;
                Some(Arc::new(LuaTerrainGenerator::new(info, source, section.script.clone(), limits)))
            }
            _ => None,
        };
        let theme = manifest.theme.as_deref().map(|t| inside(dir, t)).transpose()?;
        Ok(LoadedPlugin { manifest, dir: dir.to_owned(), generator, theme })
    }

    /// The plugin with `id`.
    pub fn get(&self, id: &str) -> Option<&LoadedPlugin> {
        self.plugins.iter().find(|p| p.manifest.id == id)
    }
}
