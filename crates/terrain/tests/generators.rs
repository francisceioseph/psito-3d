//! The built-in generators through the public API.

use psito3d_scene::{ParamValue, ParamValues, TerrainSettings, TerrainSource};
use psito3d_terrain::{CancelToken, TerrainError, TerrainGenerator, builtin_generators, generate_terrain};

fn settings(id: &str, seed: u64) -> TerrainSettings {
    TerrainSettings {
        source: TerrainSource::Generator { id: id.into(), params: ParamValues::new() },
        seed,
        resolution: 64,
        ..Default::default()
    }
}

fn run(s: &TerrainSettings) -> Result<psito3d_terrain::Heightfield, TerrainError> {
    let gens = builtin_generators();
    generate_terrain(
        s,
        |id| gens.iter().find(|g| g.info().id == id).map(|g| g.as_ref() as &dyn TerrainGenerator),
        None,
        &CancelToken::new(),
    )
}

#[test]
fn every_builtin_makes_a_varied_terrain_in_range() {
    for g in builtin_generators() {
        let field = run(&settings(&g.info().id, 7)).unwrap();
        let (lo, hi) = field.range();
        assert!(lo >= 0.0 && hi <= 1.0, "{}: {lo}..{hi}", g.info().id);
        assert!(hi - lo > 0.1, "{} is too flat", g.info().id);
        assert_eq!(field.get(0, 0), 0.0, "{}: the border slopes to the floor", g.info().id);
    }
}

#[test]
fn same_seed_same_terrain_other_seed_other_terrain() {
    let a = run(&settings("psito.island", 1)).unwrap();
    assert_eq!(a, run(&settings("psito.island", 1)).unwrap());
    assert_ne!(a, run(&settings("psito.island", 2)).unwrap());
}

#[test]
fn parameters_change_the_result() {
    let mut s = settings("psito.hills", 3);
    let low = run(&s).unwrap();
    if let TerrainSource::Generator { params, .. } = &mut s.source {
        params.insert("height".into(), ParamValue::Number(1.0));
    }
    let high = run(&s).unwrap();
    assert!(high.range().1 > low.range().1);
}

#[test]
fn unknown_generator_is_an_error() {
    assert_eq!(run(&settings("nope", 1)).unwrap_err(), TerrainError::UnknownGenerator("nope".into()));
}

#[test]
fn cancelled_before_start() {
    let gens = builtin_generators();
    let cancel = CancelToken::new();
    cancel.cancel();
    let r = generate_terrain(&settings("psito.hills", 1), |_| Some(gens[0].as_ref()), None, &cancel);
    assert_eq!(r.unwrap_err(), TerrainError::Cancelled);
}

#[test]
fn a_picture_becomes_the_terrain() {
    let dir = tempfile::tempdir().unwrap();
    let mut img = image::GrayImage::new(32, 32);
    for (x, y, p) in img.enumerate_pixels_mut() {
        let d = ((x as f32 - 16.0).powi(2) + (y as f32 - 16.0).powi(2)).sqrt();
        p.0 = [(255.0 * (1.0 - d / 16.0).max(0.0)) as u8];
    }
    img.save(dir.path().join("hill.png")).unwrap();
    let s = TerrainSettings {
        source: TerrainSource::Image { path: "hill.png".into() },
        resolution: 32,
        edge_falloff: 0.0,
        ..Default::default()
    };
    let field = generate_terrain(&s, |_| None, Some(dir.path()), &CancelToken::new()).unwrap();
    assert!(field.sample(0.0, 0.0) > 0.9 && field.get(0, 0) < 0.05);
}
