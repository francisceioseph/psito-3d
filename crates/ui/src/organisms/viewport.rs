//! The rendered image, and the camera controls on it.

use egui::{Sense, TextureId, Ui, vec2};
use psito3d_scene::Camera;

use crate::theme;

/// The brush touching the image.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaintEvent {
    /// Where, from -1 to 1 left to right and bottom to top, as
    /// [`Camera::ray`] takes it.
    pub ndc: (f32, f32),
    /// The image's width over its height.
    pub aspect: f32,
    /// Whether this is the first touch of a stroke.
    pub started: bool,
}

/// What happened in the viewport this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportResponse {
    /// The size the image should be rendered at, in physical pixels.
    pub pixels: (u32, u32),
    /// Whether the person is moving the camera right now.
    pub interacting: bool,
    /// The brush, when painting.
    pub paint: Option<PaintEvent>,
}

/// Shows `texture` filling the space, and turns drags into camera moves:
/// drag to turn around the target, right-drag (or Shift-drag) to slide,
/// scroll to come closer. With `painting` on, the left button paints
/// instead, the right one turns and the middle one slides. The image is
/// named `label` for screen readers.
pub fn viewport(
    ui: &mut Ui,
    texture: Option<TextureId>,
    camera: &mut Camera,
    label: &str,
    painting: bool,
) -> ViewportResponse {
    let t = theme::current(ui.ctx());
    let size = ui.available_size().max(vec2(64.0, 64.0));
    let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Image, true, label));

    let mut paint = None;
    if painting {
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        }
        let primary_down = ui.input(|i| i.pointer.primary_down());
        if primary_down
            && (response.is_pointer_button_down_on() || response.dragged_by(egui::PointerButton::Primary))
            && let Some(pos) = response.interact_pointer_pos()
        {
            let rel = (pos - rect.min) / rect.size();
            paint = Some(PaintEvent {
                ndc: (rel.x * 2.0 - 1.0, 1.0 - rel.y * 2.0),
                aspect: rect.width() / rect.height(),
                started: ui.input(|i| i.pointer.primary_pressed()),
            });
        }
    }

    let delta = response.drag_delta();
    let (orbit_button, pan_button) = if painting {
        (egui::PointerButton::Secondary, egui::PointerButton::Middle)
    } else {
        (egui::PointerButton::Primary, egui::PointerButton::Secondary)
    };
    let pan = response.dragged_by(pan_button)
        || response.dragged_by(egui::PointerButton::Middle)
        || (response.dragged_by(orbit_button) && ui.input(|i| i.modifiers.shift));
    let moving = response.dragged_by(orbit_button)
        || response.dragged_by(pan_button)
        || response.dragged_by(egui::PointerButton::Middle);
    if moving && delta != egui::Vec2::ZERO {
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
        interacting: moving || zoomed,
        paint,
    }
}
