//! The plugins that ship with the program load and run.

use std::path::PathBuf;

use psito3d_plugins::{Limits, PluginRegistry};
use psito3d_scene::resolve_params;
use psito3d_terrain::{CancelToken, GenerateRequest, TerrainGenerator};

fn bundled() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../plugins")
}

#[test]
fn every_bundled_plugin_loads_and_generates() {
    let reg = PluginRegistry::scan(&[bundled()], Limits::default());
    assert!(reg.errors.is_empty(), "{:?}", reg.errors);
    assert!(!reg.plugins.is_empty());
    for plugin in &reg.plugins {
        let Some(g) = &plugin.generator else { continue };
        let req =
            GenerateRequest { seed: 1, resolution: 48, params: resolve_params(&g.info().params, &Default::default()) };
        let field = g.generate(&req, &CancelToken::new()).unwrap();
        let (lo, hi) = field.range();
        assert!(hi - lo > 0.1, "{} is flat", plugin.manifest.id);
    }
}
