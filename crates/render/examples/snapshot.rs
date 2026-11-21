//! Renders the starter scene (or a `.psito3d` file) to a PNG, without a window.
//!
//! ```sh
//! cargo run --release -p psito3d-render --example snapshot -- out.png [scene.psito3d] [samples]
//! ```

use psito3d_render::{SceneRenderer, headless};
use psito3d_scene::Scene;
use psito3d_terrain::{CancelToken, TerrainGenerator, builtin_generators, generate_terrain};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let out = args.next().unwrap_or_else(|| "snapshot.png".into());
    let scene = match args.next() {
        Some(path) => Scene::from_json(&std::fs::read_to_string(path)?)?,
        None => Scene::starter(),
    };
    let samples: u32 = args.next().map_or(Ok(64), |s| s.parse())?;
    let (w, h) = (960, 540);

    let gens = builtin_generators();
    let field = generate_terrain(
        &scene.terrain,
        |id| gens.iter().find(|g| g.info().id == id).map(|g| g.as_ref() as &dyn TerrainGenerator),
        None,
        &CancelToken::new(),
    )?;
    let (device, queue) = headless::open_gpu()?;
    let mut r = SceneRenderer::new(&device, &queue, (w, h));
    r.set_heightfield(&device, &queue, &field);
    r.set_scene(&scene);
    let start = std::time::Instant::now();
    let pixels = r.render_to_pixels(&device, &queue, samples)?;
    eprintln!("{samples} samples in {:.2?}", start.elapsed());
    image::save_buffer(&out, &pixels, w, h, image::ColorType::Rgba8)?;
    Ok(())
}
