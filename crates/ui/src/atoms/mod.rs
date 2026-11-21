//! Atoms: the smallest widgets. They know nothing about scenes; they draw
//! with the theme tokens and report what the person did.
//!
//! Every atom has a name for screen readers (an accesskit label), which is
//! also how the UI tests find it.

mod button;
mod icon;
mod slider;
mod swatch;
mod text;
mod toggle;

pub use button::{Button, ButtonKind};
pub use icon::{Icon, IconButton, paint_icon};
pub use slider::Slider;
pub use swatch::Swatch;
pub use text::{caption, heading, value_text};
pub use toggle::Toggle;

use egui::{Color32, CornerRadius, Painter, Rect, Stroke, StrokeKind};

use crate::theme::Theme;

/// Paints a raised (or, when `pressed`, sunken) bevelled box, the look of
/// every button in the bevel style.
pub(crate) fn paint_bevel(
    painter: &Painter,
    rect: Rect,
    theme: &Theme,
    fill: Color32,
    radius: CornerRadius,
    pressed: bool,
) {
    painter.rect_filled(rect, radius, fill);
    let w = theme.metrics.bevel;
    if w <= 0.0 {
        painter.rect_stroke(rect, radius, Stroke::new(theme.metrics.stroke, theme.c(|p| p.line)), StrokeKind::Inside);
        return;
    }
    let (top, bottom) = if pressed {
        (theme.c(|p| p.bevel_dark), theme.c(|p| p.bevel_light))
    } else {
        (theme.c(|p| p.bevel_light), theme.c(|p| p.bevel_dark))
    };
    let r = f32::from(radius.nw);
    let inner = rect.shrink(w * 0.5);
    // The top and left edges catch the light; the bottom and right are in shade.
    painter.line_segment(
        [inner.left_top() + egui::vec2(r, 0.0), inner.right_top() - egui::vec2(r, 0.0)],
        Stroke::new(w, top),
    );
    painter.line_segment(
        [inner.left_top() + egui::vec2(0.0, r), inner.left_bottom() - egui::vec2(0.0, r)],
        Stroke::new(w, top),
    );
    painter.line_segment(
        [inner.left_bottom() + egui::vec2(r, 0.0), inner.right_bottom() - egui::vec2(r, 0.0)],
        Stroke::new(w, bottom),
    );
    painter.line_segment(
        [inner.right_top() + egui::vec2(0.0, r), inner.right_bottom() - egui::vec2(0.0, r)],
        Stroke::new(w, bottom),
    );
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(theme.metrics.stroke * 0.5, theme.c(|p| p.bevel_dark)),
        StrokeKind::Inside,
    );
}

/// The corner radius of a control of `height` for the theme's button style.
pub(crate) fn control_radius(theme: &Theme, height: f32) -> CornerRadius {
    match theme.components.button {
        crate::theme::ButtonStyle::Pill => CornerRadius::same((height * 0.5).round() as u8),
        _ => CornerRadius::same(theme.metrics.radius.round() as u8),
    }
}
