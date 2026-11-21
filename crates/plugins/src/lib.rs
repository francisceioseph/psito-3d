//! The Lua plugin host of Psito 3D.
//!
//! A plugin is a folder with a `plugin.toml` manifest and the files it
//! names. The manifest is read without running any code: it says what the
//! plugin provides (a terrain generator, a theme) and describes the
//! parameters, so the editor can show the controls before a line of Lua runs.
//!
//! Plugin code runs in a [sandbox](sandbox): Lua 5.4 with only the `math`,
//! `string`, `table` and `utf8` libraries, no way to load other code or touch
//! files, a memory limit and a time limit. What a plugin can do is exactly the
//! [`psito` API](api), documented in `docs/PLUGINS.md`.

pub mod api;
mod lua_generator;
mod manifest;
mod registry;
pub mod sandbox;

pub use lua_generator::LuaTerrainGenerator;
pub use manifest::{API_VERSION, PluginManifest, TerrainSection};
pub use registry::{LoadedPlugin, PluginError, PluginRegistry};
pub use sandbox::Limits;
