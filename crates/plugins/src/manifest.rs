//! The `plugin.toml` manifest.

use psito3d_scene::{LocalizedText, ParamSpec};
use psito3d_terrain::GeneratorInfo;
use serde::Deserialize;

/// The version of the plugin API this program offers. A plugin asks for a
/// version in its manifest and is refused if it asks for a newer one.
pub const API_VERSION: u32 = 1;

/// The `[terrain]` section: the plugin makes terrains.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainSection {
    /// The Lua file, relative to the plugin folder.
    pub script: String,
    /// The parameters, in the order shown.
    #[serde(default)]
    pub params: Vec<ParamSpec>,
}

/// The manifest of a plugin.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginManifest {
    /// A reverse domain name, such as `org.example.volcano`. Scenes store it.
    pub id: String,
    /// The name shown in the editor.
    pub name: LocalizedText,
    /// One sentence on what it does.
    #[serde(default)]
    pub description: LocalizedText,
    /// The plugin's own version, for people.
    pub version: String,
    /// Who made it.
    #[serde(default)]
    pub author: Option<String>,
    /// The plugin API version it was written for.
    pub api: u32,
    /// Present when the plugin makes terrains.
    #[serde(default)]
    pub terrain: Option<TerrainSection>,
    /// A theme pack (`theme.toml`), relative to the plugin folder.
    #[serde(default)]
    pub theme: Option<String>,
}

impl PluginManifest {
    /// Reads a manifest. Unknown keys are refused, so a typo is noticed.
    pub fn from_toml(text: &str) -> Result<Self, String> {
        let m: Self = toml::from_str(text).map_err(|e| e.to_string())?;
        if m.api > API_VERSION {
            return Err(format!("needs plugin API {} and this program offers {API_VERSION}", m.api));
        }
        if m.api == 0 {
            return Err("api must be 1 or more".into());
        }
        if m.terrain.is_none() && m.theme.is_none() {
            return Err("the plugin provides nothing: add [terrain] or theme".into());
        }
        if let Some(info) = m.generator_info() {
            info.validate().map_err(|e| e.to_string())?;
        } else {
            // Still check the id, which every plugin has.
            GeneratorInfo { id: m.id.clone(), name: m.name.clone(), description: Default::default(), params: vec![] }
                .validate()
                .map_err(|e| e.to_string())?;
        }
        Ok(m)
    }

    /// The description of the terrain generator, if the plugin has one. Its
    /// id is the plugin id.
    pub fn generator_info(&self) -> Option<GeneratorInfo> {
        self.terrain.as_ref().map(|t| GeneratorInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            params: t.params.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"
        id = "org.example.volcano"
        name = { en = "Volcano", pt = "Vulcão" }
        version = "1.0.0"
        api = 1
        [terrain]
        script = "terrain.lua"
        [[terrain.params]]
        id = "crater"
        label = "Crater"
        kind = "slider"
        min = 0.0
        max = 0.5
        default = 0.15
    "#;

    #[test]
    fn reads_a_good_manifest() {
        let m = PluginManifest::from_toml(GOOD).unwrap();
        let info = m.generator_info().unwrap();
        assert_eq!(info.id, "org.example.volcano");
        assert_eq!(info.params[0].id, "crater");
        assert_eq!(m.name.get("pt"), "Vulcão");
    }

    #[test]
    fn refuses_a_newer_api() {
        let text = GOOD.replace("api = 1", "api = 99");
        assert!(PluginManifest::from_toml(&text).unwrap_err().contains("API 99"));
    }

    #[test]
    fn refuses_typos_and_empty_plugins() {
        assert!(PluginManifest::from_toml(&GOOD.replace("version", "verison")).is_err());
        let empty = "id = \"a.b\"\nname = \"A\"\nversion = \"1\"\napi = 1\n";
        assert!(PluginManifest::from_toml(empty).unwrap_err().contains("provides nothing"));
    }

    #[test]
    fn refuses_bad_params() {
        let text = GOOD.replace("default = 0.15", "default = 9.0");
        assert!(PluginManifest::from_toml(&text).is_err());
    }
}
