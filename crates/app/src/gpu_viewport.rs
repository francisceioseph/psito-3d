//! The renderer inside the eframe window: shares eframe's GPU device and
//! hands the image to egui as a texture.

use eframe::egui_wgpu::RenderState;
use psito3d_render::{Quality, SceneRenderer};
use psito3d_scene::Scene;
use psito3d_terrain::Heightfield;

/// The scene renderer and its egui texture.
pub struct GpuViewport {
    renderer: SceneRenderer,
    texture: egui::TextureId,
    generation: u64,
}

impl GpuViewport {
    /// A renderer on eframe's device.
    pub fn new(rs: &RenderState) -> Self {
        let renderer = SceneRenderer::new(&rs.device, &rs.queue, (64, 64));
        let texture =
            rs.renderer.write().register_native_texture(&rs.device, renderer.output_view(), wgpu::FilterMode::Linear);
        Self { generation: renderer.output_generation(), renderer, texture }
    }

    /// The texture to show.
    pub fn texture(&self) -> egui::TextureId {
        self.texture
    }

    /// Replaces the terrain.
    pub fn set_heightfield(&mut self, rs: &RenderState, field: &Heightfield) {
        self.renderer.set_heightfield(&rs.device, &rs.queue, field);
    }

    /// Draws one more sample of `scene` at `size` pixels, if it needs one.
    /// Returns whether more samples are wanted (to keep repainting).
    pub fn frame(&mut self, rs: &RenderState, scene: &Scene, size: (u32, u32), quality: Quality) -> bool {
        self.renderer.resize(&rs.device, size);
        self.renderer.set_quality(&rs.device, quality);
        self.renderer.set_scene(scene);
        self.renderer.render(&rs.device, &rs.queue);
        if self.renderer.output_generation() != self.generation {
            self.generation = self.renderer.output_generation();
            rs.renderer.write().update_egui_texture_from_wgpu_texture(
                &rs.device,
                self.renderer.output_view(),
                wgpu::FilterMode::Linear,
                self.texture,
            );
        }
        self.renderer.needs_samples()
    }

    /// Samples so far and samples wanted.
    pub fn progress(&self) -> (u32, u32) {
        (self.renderer.samples(), self.renderer.target_samples())
    }
}
