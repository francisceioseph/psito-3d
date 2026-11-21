//! Renders real images on whatever GPU is available (llvmpipe works).
//!
//! These are the slowest tests, so they stay few and small. Without any
//! adapter they print a note and pass, so a machine with no GPU at all can
//! still run the rest of the suite; set `PSITO3D_REQUIRE_GPU=1` to make that
//! a failure instead (the CI does).

use psito3d_render::{SceneRenderer, headless};
use psito3d_scene::{Filter, Material, Scene, Shape, Sun};
use psito3d_terrain::{CancelToken, TerrainGenerator, builtin_generators, generate_terrain};

fn gpu() -> Option<(wgpu::Device, wgpu::Queue)> {
    match headless::open_gpu() {
        Ok(gpu) => Some(gpu),
        Err(e) if std::env::var_os("PSITO3D_REQUIRE_GPU").is_none() => {
            eprintln!("skipping: {e}");
            None
        }
        Err(e) => panic!("{e}"),
    }
}

const W: u32 = 96;
const H: u32 = 64;

fn render(scene: &Scene, samples: u32) -> Option<Vec<u8>> {
    let (device, queue) = gpu()?;
    let gens = builtin_generators();
    let mut terrain = scene.terrain.clone();
    terrain.resolution = 128;
    let field = generate_terrain(
        &terrain,
        |id| gens.iter().find(|g| g.info().id == id).map(|g| g.as_ref() as &dyn TerrainGenerator),
        None,
        &CancelToken::new(),
    )
    .unwrap();
    let mut r = SceneRenderer::new(&device, &queue, (W, H));
    r.set_heightfield(&device, &queue, &field);
    r.set_scene(scene);
    Some(r.render_to_pixels(&device, &queue, samples).unwrap())
}

fn pixel(img: &[u8], x: u32, y: u32) -> [u8; 3] {
    let i = ((y * W + x) * 4) as usize;
    [img[i], img[i + 1], img[i + 2]]
}

fn luma(p: [u8; 3]) -> f32 {
    0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
}

fn looking_at_the_horizon() -> Scene {
    let mut s = Scene::default();
    s.camera.target = glam::Vec3::new(0.0, 5.0, 0.0);
    s.camera.pitch = 2.0;
    s.camera.distance = 200.0;
    s.sky.cloud_cover = 0.0;
    s.fog.density = 0.0;
    s.fog.mist = 0.0;
    s
}

#[test]
fn daytime_sky_is_blue_and_bright() {
    let mut scene = looking_at_the_horizon();
    scene.sky.sun = Sun { azimuth: 180.0, elevation: 50.0 };
    let Some(img) = render(&scene, 2) else { return };
    let top = pixel(&img, W / 2, 1);
    assert!(top[2] > top[0], "the sky at the top is blue: {top:?}");
    assert!(luma(top) > 60.0, "and bright: {top:?}");
}

#[test]
fn night_is_dark() {
    let mut scene = looking_at_the_horizon();
    scene.sky.sun = Sun { azimuth: 180.0, elevation: -40.0 };
    let Some(img) = render(&scene, 2) else { return };
    assert!(luma(pixel(&img, W / 2, 1)) < 30.0);
}

#[test]
fn a_red_ball_shows_in_the_middle() {
    let mut scene = looking_at_the_horizon();
    scene.sea.enabled = false;
    // High above the hills, so the ball is not buried.
    scene.camera.target.y = 60.0;
    let target = scene.camera.target;
    let id = scene.add_object(Shape::Sphere, Material::clay([0.9, 0.05, 0.05])).unwrap();
    let ball = scene.object_mut(id).unwrap();
    ball.transform.position = target;
    ball.transform.scale = glam::Vec3::splat(40.0);
    let Some(img) = render(&scene, 2) else { return };
    let c = pixel(&img, W / 2, H / 2);
    assert!(c[0] > c[1] + 20 && c[0] > c[2] + 20, "the red ball fills the centre: {c:?}");
}

#[test]
fn accumulating_reduces_noise() {
    // Rough glass is the noisiest material: each sample picks one path.
    let mut scene = looking_at_the_horizon();
    let target = scene.camera.target;
    let id = scene.add_object(Shape::Sphere, Material { roughness: 0.4, ..Material::glass() }).unwrap();
    let ball = scene.object_mut(id).unwrap();
    ball.transform.position = target;
    ball.transform.scale = glam::Vec3::splat(60.0);
    let (Some(a1), Some(b1), Some(a32), Some(b32)) =
        (render(&scene, 1), render(&scene, 1), render(&scene, 24), render(&scene, 24))
    else {
        return;
    };
    let diff = |a: &[u8], b: &[u8]| a.iter().zip(b).map(|(x, y)| (*x as i32 - *y as i32).abs()).sum::<i32>();
    // The same seed gives the same image, so compare one sample with many.
    assert_eq!(diff(&a1, &b1), 0, "rendering is deterministic");
    assert_eq!(diff(&a32, &b32), 0);
    assert_ne!(diff(&a1, &a32), 0, "more samples change the image");
}

#[test]
fn the_dither_filter_limits_the_colours() {
    let mut scene = Scene::starter();
    scene.filter = Filter::Dither;
    let Some(img) = render(&scene, 1) else { return };
    let colours: std::collections::BTreeSet<[u8; 3]> = img.chunks(4).map(|p| [p[0], p[1], p[2]]).collect();
    assert!(colours.len() <= 216, "{} colours", colours.len());
}
