//! Where the program keeps things on disk.

use std::path::PathBuf;

const APP_DIR: &str = "psito-3d";

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from)
}

/// The folder of the preferences: `$XDG_CONFIG_HOME/psito-3d` (or
/// `~/.config/psito-3d`), or `%APPDATA%\psito-3d` on Windows.
pub fn config_dir() -> Option<PathBuf> {
    if cfg!(windows) {
        return std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join(APP_DIR));
    }
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| home().map(|h| h.join(".config")))
        .map(|d| d.join(APP_DIR))
}

/// The folder of the person's own plugins: `$XDG_DATA_HOME/psito-3d/plugins`
/// (or `~/.local/share/...`), or `%APPDATA%\psito-3d\plugins` on Windows.
pub fn user_plugin_dir() -> Option<PathBuf> {
    if cfg!(windows) {
        return std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join(APP_DIR).join("plugins"));
    }
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| home().map(|h| h.join(".local/share")))
        .map(|d| d.join(APP_DIR).join("plugins"))
}

/// Every folder plugins are looked for in, the person's own first, so they
/// can replace a bundled plugin with their version.
pub fn plugin_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = user_plugin_dir().into_iter().collect();
    if let Some(exe_dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.to_owned())) {
        dirs.push(exe_dir.join("plugins"));
        // Installed on Linux: /usr/bin/psito-3d → /usr/share/psito-3d/plugins.
        dirs.push(exe_dir.join("../share/psito-3d/plugins"));
    }
    if cfg!(debug_assertions) {
        dirs.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../plugins"));
    }
    dirs
}
