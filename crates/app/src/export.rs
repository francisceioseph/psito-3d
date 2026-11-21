//! Saving a picture: a separate renderer at the picture's size, a few
//! samples per frame so the window stays responsive, then a PNG written on
//! another thread.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, channel};

use eframe::egui_wgpu::RenderState;
use psito3d_render::SceneRenderer;
use psito3d_scene::Scene;
use psito3d_terrain::Heightfield;

/// The picture size and quality of an export.
pub const EXPORT_SIZE: (u32, u32) = (1920, 1080);
/// The samples of an exported picture.
pub const EXPORT_SAMPLES: u32 = 512;
const SAMPLES_PER_FRAME: u32 = 4;

/// An export in progress.
pub struct ExportJob {
    renderer: SceneRenderer,
    path: PathBuf,
    writing: Option<Receiver<Result<(), String>>>,
}

impl ExportJob {
    /// Starts rendering `scene` on `field` for `path`.
    pub fn start(rs: &RenderState, scene: &Scene, field: &Heightfield, path: PathBuf) -> Self {
        let mut renderer = SceneRenderer::new(&rs.device, &rs.queue, EXPORT_SIZE);
        renderer.set_heightfield(&rs.device, &rs.queue, field);
        renderer.set_scene(scene);
        renderer.set_target_samples(EXPORT_SAMPLES);
        Self { renderer, path, writing: None }
    }

    /// Does a little more work. Returns `Some` with the outcome when done.
    pub fn step(&mut self, rs: &RenderState) -> Option<Result<PathBuf, String>> {
        if let Some(rx) = &self.writing {
            return rx.try_recv().ok().map(|r| r.map(|()| self.path.clone()));
        }
        for _ in 0..SAMPLES_PER_FRAME {
            if !self.renderer.needs_samples() {
                break;
            }
            self.renderer.render(&rs.device, &rs.queue);
        }
        if self.renderer.needs_samples() {
            return None;
        }
        self.renderer.render(&rs.device, &rs.queue);
        let pixels = match self.renderer.read_output(&rs.device, &rs.queue) {
            Ok(p) => p,
            Err(e) => return Some(Err(e.to_string())),
        };
        let (tx, rx) = channel();
        let path = self.path.clone();
        let (w, h) = EXPORT_SIZE;
        std::thread::spawn(move || {
            let result = image::save_buffer(&path, &pixels, w, h, image::ColorType::Rgba8).map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
        self.writing = Some(rx);
        None
    }

    /// Samples done so far, out of [`EXPORT_SAMPLES`].
    pub fn progress(&self) -> f32 {
        self.renderer.samples() as f32 / EXPORT_SAMPLES as f32
    }
}
