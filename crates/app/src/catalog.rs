//! Everything that can be picked: terrain generators and themes, built in
//! and from plugins.

use std::path::PathBuf;
use std::sync::Arc;

use psito3d_plugins::{Limits, PluginRegistry};
use psito3d_terrain::{TerrainGenerator, builtin_generators};
use psito3d_ui::organisms::GeneratorChoice;
use psito3d_ui::theme::{Theme, builtin_themes};

/// The generators, shared with the terrain worker thread.
pub type Generators = Arc<Vec<Arc<dyn TerrainGenerator>>>;

/// The generators and themes on offer, and the plugins that failed.
pub struct Catalog {
    /// Built-in generators first, then the plugins' ones.
    pub generators: Generators,
    /// Built-in themes first, then the plugins' ones.
    pub themes: Vec<Theme>,
    /// One line per plugin (or plugin theme) that did not load.
    pub errors: Vec<String>,
}

impl Catalog {
    /// The built-in generators and themes, and the plugins found in `dirs`.
    pub fn load(dirs: &[PathBuf]) -> Self {
        let registry = PluginRegistry::scan(dirs, Limits::default());
        let mut generators = builtin_generators();
        let mut themes = builtin_themes();
        let mut errors: Vec<String> = registry.errors.iter().map(ToString::to_string).collect();
        for plugin in &registry.plugins {
            if let Some(g) = &plugin.generator {
                if generators.iter().any(|b| b.info().id == g.info().id) {
                    errors.push(format!("{}: the id {} is taken", plugin.dir.display(), g.info().id));
                } else {
                    generators.push(Arc::clone(g) as Arc<dyn TerrainGenerator>);
                }
            }
            if let Some(path) = &plugin.theme {
                match Theme::from_file(path) {
                    Ok(t) if themes.iter().any(|b| b.id == t.id) => {
                        errors.push(format!("{}: the theme id {} is taken", path.display(), t.id));
                    }
                    Ok(t) => themes.push(t),
                    Err(e) => errors.push(format!("{}: {e}", path.display())),
                }
            }
        }
        for e in &errors {
            log::warn!("plugin: {e}");
        }
        log::info!(
            "plugins: {} loaded ({}), {} failed",
            registry.plugins.len(),
            registry.plugins.iter().map(|p| p.manifest.id.as_str()).collect::<Vec<_>>().join(", "),
            errors.len()
        );
        Self { generators: Arc::new(generators), themes, errors }
    }

    /// The generator with `id`.
    pub fn generator(&self, id: &str) -> Option<&dyn TerrainGenerator> {
        self.generators.iter().find(|g| g.info().id == id).map(|g| g.as_ref())
    }

    /// The generators as the terrain panel shows them, in `lang`.
    pub fn choices(&self, lang: &str) -> Vec<GeneratorChoice> {
        self.generators
            .iter()
            .map(|g| {
                let info = g.info();
                GeneratorChoice {
                    id: info.id.clone(),
                    name: info.name.get(lang).to_owned(),
                    description: info.description.get(lang).to_owned(),
                    params: info.params.clone(),
                }
            })
            .collect()
    }

    /// The themes as the toolbar shows them: id and name in `lang`.
    pub fn theme_names(&self, lang: &str) -> Vec<(String, String)> {
        self.themes.iter().map(|t| (t.id.clone(), t.name.get(lang).to_owned())).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundled() -> Catalog {
        Catalog::load(&[PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../plugins")])
    }

    #[test]
    fn bundled_plugins_join_the_built_ins() {
        let c = bundled();
        assert!(c.errors.is_empty(), "{:?}", c.errors);
        assert!(c.generator("psito.hills").is_some());
        assert!(c.generator("org.psito.volcanic-island").is_some());
        assert!(c.themes.iter().any(|t| t.id == "org.psito.vaporwave"));
    }

    #[test]
    fn names_follow_the_language() {
        let c = bundled();
        let pt: Vec<String> = c.choices("pt").into_iter().map(|g| g.name).collect();
        assert!(pt.contains(&"Ilha vulcânica".to_owned()) && pt.contains(&"Colinas".to_owned()));
        assert!(c.theme_names("pt").iter().any(|(_, n)| n == "Entardecer"));
    }

    #[test]
    fn missing_folders_are_fine() {
        let c = Catalog::load(&[PathBuf::from("/nonexistent/psito-3d")]);
        assert!(c.errors.is_empty());
        assert_eq!(c.generators.len(), builtin_generators().len());
    }
}
