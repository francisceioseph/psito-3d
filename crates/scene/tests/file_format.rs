//! The `.psito3d` file format, read and written through the public API.

use psito3d_scene::{FORMAT_VERSION, Scene, SceneError, Shape};

#[test]
fn round_trip_keeps_everything() {
    let scene = Scene::starter();
    let back = Scene::from_json(&scene.to_json()).unwrap();
    assert_eq!(back, scene);
}

#[test]
fn missing_fields_take_defaults() {
    let scene = Scene::from_json(r#"{ "sky": { "haze": 0.7 } }"#).unwrap();
    assert_eq!(scene.sky.haze, 0.7);
    assert_eq!(scene.sea, Default::default());
    assert_eq!(scene.format, FORMAT_VERSION);
}

#[test]
fn out_of_range_values_are_clamped() {
    let scene = Scene::from_json(
        r#"{ "sea": { "level": 7 }, "terrain": { "resolution": 99999 },
             "objects": [ { "id": 9, "name": "a", "shape": "sphere", "blend": -3 } ] }"#,
    )
    .unwrap();
    assert_eq!(scene.sea.level, 1.0);
    assert_eq!(scene.terrain.resolution, 2048);
    assert_eq!(scene.objects[0].blend, 0.0);
    assert!(scene.next_object_id > 9, "new ids never repeat one in the file");
}

#[test]
fn newer_files_are_refused() {
    let err = Scene::from_json(r#"{ "format": 99 }"#).unwrap_err();
    assert!(matches!(err, SceneError::TooNew { found: 99 }));
}

#[test]
fn garbage_is_refused() {
    assert!(matches!(Scene::from_json("not json"), Err(SceneError::Parse(_))));
}

#[test]
fn object_list_has_a_limit() {
    let mut scene = Scene::default();
    let added = (0..100).filter_map(|_| scene.add_object(Shape::Cube, Default::default())).count();
    assert_eq!(added, 64);
    let first = scene.objects[0].id;
    assert!(scene.remove_object(first));
    assert!(scene.object(first).is_none());
}
