//! Rendering a picture apart from the viewport: a separate renderer at the
//! picture's size, a few samples per frame so the window stays responsive,
//! then the PNG written (and, for a wallpaper, put on the desktop) on
//! another thread.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};

use eframe::egui_wgpu::RenderState;
use psito3d_render::SceneRenderer;
use psito3d_scene::Scene;
use psito3d_terrain::Heightfield;

use crate::wallpaper::{WallpaperError, WallpaperSetter};

const SAMPLES_PER_FRAME: u32 = 4;

/// Size and quality of a picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PictureSettings {
    /// Width and height in pixels.
    pub size: (u32, u32),
    /// Samples per pixel.
    pub samples: u32,
}

impl PictureSettings {
    /// A saved picture: Full HD, very clean.
    pub const EXPORT: Self = Self { size: (1920, 1080), samples: 512 };
    /// A wallpaper, when the screen size is not known.
    pub const WALLPAPER: Self = Self { size: (1920, 1080), samples: 256 };
    /// The widest wallpaper made, so a huge screen does not take forever.
    pub const MAX_WALLPAPER_WIDTH: u32 = 3840;

    /// The wallpaper settings for a screen of `pixels`, keeping its shape.
    pub fn wallpaper_for(pixels: Option<(u32, u32)>, base: Self) -> Self {
        let Some((w, h)) = pixels.filter(|(w, h)| *w >= 64 && *h >= 64) else { return base };
        let scale = (Self::MAX_WALLPAPER_WIDTH as f32 / w as f32).min(1.0);
        Self { size: (((w as f32) * scale) as u32, ((h as f32) * scale) as u32), ..base }
    }
}

/// What happens to the picture when it is ready.
#[derive(Debug, Clone)]
pub enum Purpose {
    /// It is saved where the person chose.
    File,
    /// It is saved and put on the desktop.
    Wallpaper(Arc<dyn WallpaperSetter>),
}

/// How a finished job went.
#[derive(Debug)]
pub enum Outcome {
    /// The picture was saved.
    Saved(PathBuf),
    /// The picture was saved and is now the wallpaper.
    WallpaperSet(PathBuf),
    /// The picture was saved, but the wallpaper could not be changed.
    WallpaperFailed(PathBuf, WallpaperError),
    /// The picture could not be made or saved.
    Failed(String),
}

/// A picture being rendered.
pub struct ExportJob {
    renderer: SceneRenderer,
    path: PathBuf,
    settings: PictureSettings,
    purpose: Purpose,
    writing: Option<Receiver<Outcome>>,
}

impl ExportJob {
    /// Starts rendering `scene` on `field` into `path`.
    pub fn start(
        rs: &RenderState,
        scene: &Scene,
        field: &Heightfield,
        path: PathBuf,
        settings: PictureSettings,
        purpose: Purpose,
    ) -> Self {
        let mut renderer = SceneRenderer::new(&rs.device, &rs.queue, settings.size);
        renderer.set_heightfield(&rs.device, &rs.queue, field);
        renderer.set_scene(scene);
        renderer.set_target_samples(settings.samples);
        Self { renderer, path, settings, purpose, writing: None }
    }

    /// Whether this job makes a wallpaper.
    pub fn is_wallpaper(&self) -> bool {
        matches!(self.purpose, Purpose::Wallpaper(_))
    }

    /// Does a little more work. Returns the outcome when done.
    pub fn step(&mut self, rs: &RenderState) -> Option<Outcome> {
        if let Some(rx) = &self.writing {
            return rx.try_recv().ok();
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
            Err(e) => return Some(Outcome::Failed(e.to_string())),
        };
        let (tx, rx) = channel();
        let path = self.path.clone();
        let (w, h) = self.settings.size;
        let purpose = self.purpose.clone();
        std::thread::spawn(move || {
            let saved =
                path.parent().map_or(Ok(()), std::fs::create_dir_all).map_err(|e| e.to_string()).and_then(|()| {
                    image::save_buffer(&path, &pixels, w, h, image::ColorType::Rgba8).map_err(|e| e.to_string())
                });
            let outcome = match (saved, purpose) {
                (Err(e), _) => Outcome::Failed(e),
                (Ok(()), Purpose::File) => Outcome::Saved(path),
                (Ok(()), Purpose::Wallpaper(setter)) => match setter.set(&path) {
                    Ok(()) => Outcome::WallpaperSet(path),
                    Err(e) => Outcome::WallpaperFailed(path, e),
                },
            };
            let _ = tx.send(outcome);
        });
        self.writing = Some(rx);
        None
    }

    /// The fraction of the samples done.
    pub fn progress(&self) -> f32 {
        self.renderer.samples() as f32 / self.settings.samples.max(1) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wallpapers_follow_the_screen_shape_and_stay_reasonable() {
        let base = PictureSettings::WALLPAPER;
        assert_eq!(PictureSettings::wallpaper_for(None, base), base);
        assert_eq!(PictureSettings::wallpaper_for(Some((2560, 1080)), base).size, (2560, 1080));
        assert_eq!(PictureSettings::wallpaper_for(Some((7680, 4320)), base).size, (3840, 2160));
        assert_eq!(PictureSettings::wallpaper_for(Some((0, 0)), base), base);
    }
}
