//! Loading plugin folders from disk.

use std::fs;
use std::path::Path;

use psito3d_plugins::{Limits, PluginRegistry};
use psito3d_scene::{TerrainSettings, TerrainSource};
use psito3d_terrain::{CancelToken, TerrainGenerator, generate_terrain};

fn write_plugin(root: &Path, folder: &str, id: &str, script: &str, lua: &str) {
    let dir = root.join(folder);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("plugin.toml"),
        format!("id = \"{id}\"\nname = \"Test\"\nversion = \"1\"\napi = 1\n[terrain]\nscript = \"{script}\"\n"),
    )
    .unwrap();
    fs::write(dir.join("terrain.lua"), lua).unwrap();
}

#[test]
fn loads_good_plugins_and_reports_bad_ones() {
    let root = tempfile::tempdir().unwrap();
    write_plugin(root.path(), "a", "org.test.a", "terrain.lua", "return function() return 0.5 end");
    write_plugin(root.path(), "b", "org.test.b", "../../etc/passwd", "");
    write_plugin(root.path(), "c", "org.test.a", "terrain.lua", "return function() return 0 end");
    fs::create_dir_all(root.path().join("not-a-plugin")).unwrap();

    let reg = PluginRegistry::scan(&[root.path().to_owned()], Limits::default());
    assert_eq!(reg.plugins.len(), 1);
    assert_eq!(reg.errors.len(), 2);
    assert!(reg.errors.iter().any(|e| e.message.contains("inside the plugin folder")));
    assert!(reg.errors.iter().any(|e| e.message.contains("already uses the id")));
}

#[test]
fn a_plugin_generator_builds_a_terrain() {
    let root = tempfile::tempdir().unwrap();
    write_plugin(root.path(), "a", "org.test.flat", "terrain.lua", "return function() return 0.5 end");
    let reg = PluginRegistry::scan(&[root.path().to_owned()], Limits::default());
    let generator = reg.get("org.test.flat").unwrap().generator.clone().unwrap();
    let settings = TerrainSettings {
        source: TerrainSource::Generator { id: "org.test.flat".into(), params: Default::default() },
        resolution: 32,
        edge_falloff: 0.0,
        ..Default::default()
    };
    let field =
        generate_terrain(&settings, |_| Some(generator.as_ref() as &dyn TerrainGenerator), None, &CancelToken::new())
            .unwrap();
    assert_eq!(field.range(), (0.5, 0.5));
}

#[cfg(unix)]
#[test]
fn links_out_of_the_folder_are_refused() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("evil.lua"), "return function() return 1 end").unwrap();
    write_plugin(root.path(), "a", "org.test.link", "link.lua", "");
    std::os::unix::fs::symlink(outside.path().join("evil.lua"), root.path().join("a/link.lua")).unwrap();
    let reg = PluginRegistry::scan(&[root.path().to_owned()], Limits::default());
    assert!(reg.plugins.is_empty());
    assert!(reg.errors[0].message.contains("leads outside"));
}
