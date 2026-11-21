//! The rendered image, and the camera controls on it.

use egui::{Sense, TextureId, Ui, vec2};
use psito3d_scene::Camera;

use crate::theme;

/// What happened in the viewport this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportResponse {
    /// The size the image should be rendered at, in physical pixels.
    pub pixels: (u32, u32),
    /// Whether the person is moving the camera right now.
    pub interacting: bool,
}

/// Shows `texture` filling the space, and turns drags into camera moves:
/// drag to turn around the target, right-drag (or Shift-drag) to slide,
/// scroll to come closer. The image is named `label` for screen readers.
pub fn viewport(ui: &mut Ui, texture: Option<TextureId>, camera: &mut Camera, label: &str) -> ViewportResponse {
    let t = theme::current(ui.ctx());
    let size = ui.available_size().max(vec2(64.0, 64.0));
    let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Image, true, label));

    let delta = response.drag_delta();
    let pan = response.dragged_by(egui::PointerButton::Secondary)
        || response.dragged_by(egui::PointerButton::Middle)
        || (response.dragged_by(egui::PointerButton::Primary) && ui.input(|i| i.modifiers.shift));
    if response.dragged() && delta != egui::Vec2::ZERO {
        if pan {
            camera.pan(-delta.x / rect.height(), delta.y / rect.height());
        } else {
            camera.orbit(-delta.x * 0.3, delta.y * 0.2);
        }
    }
    let mut zoomed = false;
    if response.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            camera.zoom((-scroll * 0.0015).exp());
            zoomed = true;
        }
    }

    ui.painter().rect_filled(rect, 0, t.c(|p| p.sunken));
    if let Some(tex) = texture {
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        ui.painter().image(tex, rect, uv, egui::Color32::WHITE);
    }
    let ppp = ui.ctx().pixels_per_point();
    ViewportResponse {
        pixels: ((rect.width() * ppp).round() as u32, (rect.height() * ppp).round() as u32),
        interacting: response.dragged() || zoomed,
    }
}
