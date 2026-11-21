//! The person's preferences, kept between runs.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// What is remembered between runs.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    /// The id of the theme.
    pub theme: Option<String>,
    /// The language code; empty follows the system.
    pub language: Option<String>,
}

impl Prefs {
    /// The preferences file.
    pub fn path() -> Option<PathBuf> {
        crate::paths::config_dir().map(|d| d.join("prefs.json"))
    }

    /// Reads the preferences at `path`; anything wrong gives the defaults.
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
    }

    /// Writes them to `path`, creating the folder.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self).expect("serialisable"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_bad_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub/prefs.json");
        assert_eq!(Prefs::load(&path), Prefs::default());
        let prefs = Prefs { theme: Some("psito.daylight".into()), language: Some("pt".into()) };
        prefs.save(&path).unwrap();
        assert_eq!(Prefs::load(&path), prefs);
        std::fs::write(&path, "{ broken").unwrap();
        assert_eq!(Prefs::load(&path), Prefs::default());
    }
}
