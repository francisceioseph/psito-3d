//! Renders the program's icon with the program's own renderer: an island at
//! sunset with a chrome ball, in a rounded square.
//!
//! ```sh
//! cargo run --release -p psito3d-render --example icon -- assets
//! ```
//!
//! Writes `icon.png` (512 px, for the window), `icon.ico` (16 to 256 px,
//! for Windows) and `hicolor/<n>x<n>.png` (for the Linux icon theme) into
//! the folder given.

use std::path::PathBuf;

use image::{ImageBuffer, Rgba, RgbaImage, imageops::FilterType};
use psito3d_render::{SceneRenderer, headless};
use psito3d_scene::{Material, Scene, Shape, Sun, TerrainSource};
use psito3d_terrain::{CancelToken, TerrainGenerator, builtin_generators, generate_terrain};

const SIZE: u32 = 512;

fn scene() -> Scene {
    let mut s = Scene::default();
    s.terrain.source = TerrainSource::Generator { id: "psito.island".into(), params: Default::default() };
    s.terrain.seed = 7;
    s.terrain.height = 30.0;
    s.sky.sun = Sun { azimuth: 330.0, elevation: 7.0 };
    s.sky.cloud_cover = 0.5;
    s.sky.haze = 0.2;
    s.fog.density = 0.1;
    s.camera.target = glam::Vec3::new(0.0, 8.0, 0.0);
    s.camera.distance = 80.0;
    s.camera.yaw = 20.0;
    s.camera.pitch = 10.0;
    s.camera.fov = 50.0;
    let id = s.add_object(Shape::Sphere, Material::chrome()).expect("room for one");
    let ball = s.object_mut(id).expect("just added");
    ball.transform.position = glam::Vec3::new(12.0, 14.0, 22.0);
    ball.transform.scale = glam::Vec3::splat(14.0);
    s
}

/// Cuts the image into a rounded square with soft edges.
fn round_corners(img: &mut RgbaImage) {
    let n = img.width() as f32;
    let r = n * 0.22;
    for (x, y, p) in img.enumerate_pixels_mut() {
        let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
        let cx = fx.clamp(r, n - r);
        let cy = fy.clamp(r, n - r);
        let d = ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt();
        let alpha = (r - d + 0.5).clamp(0.0, 1.0);
        p.0[3] = (p.0[3] as f32 * alpha) as u8;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "assets".into()));
    std::fs::create_dir_all(&out)?;
    let scene = scene();
    let gens = builtin_generators();
    let mut terrain = scene.terrain.clone();
    terrain.resolution = 1024;
    let field = generate_terrain(
        &terrain,
        |id| gens.iter().find(|g| g.info().id == id).map(|g| g.as_ref() as &dyn TerrainGenerator),
        None,
        &CancelToken::new(),
    )?;
    let (device, queue) = headless::open_gpu()?;
    let mut r = SceneRenderer::new(&device, &queue, (SIZE, SIZE));
    r.set_heightfield(&device, &queue, &field);
    r.set_scene(&scene);
    let pixels = r.render_to_pixels(&device, &queue, 512)?;
    let mut img: RgbaImage = ImageBuffer::<Rgba<u8>, _>::from_raw(SIZE, SIZE, pixels).ok_or("bad size")?;
    round_corners(&mut img);
    img.save(out.join("icon.png"))?;

    std::fs::create_dir_all(out.join("hicolor"))?;
    for s in [48u32, 64, 128, 256, 512] {
        image::imageops::resize(&img, s, s, FilterType::Lanczos3).save(out.join(format!("hicolor/{s}x{s}.png")))?;
    }

    let frames: Vec<_> = [16u32, 24, 32, 48, 64, 128, 256]
        .into_iter()
        .map(|s| {
            let small = image::imageops::resize(&img, s, s, FilterType::Lanczos3);
            image::codecs::ico::IcoFrame::as_png(small.as_raw(), s, s, image::ExtendedColorType::Rgba8)
        })
        .collect::<Result<_, _>>()?;
    let file = std::fs::File::create(out.join("icon.ico"))?;
    image::codecs::ico::IcoEncoder::new(file).encode_images(&frames)?;
    eprintln!("wrote {}", out.display());
    Ok(())
}
