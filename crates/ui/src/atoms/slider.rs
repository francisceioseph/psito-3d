//! A horizontal slider.

use std::ops::RangeInclusive;

use egui::{CornerRadius, Key, Response, Sense, Stroke, Ui, Widget, WidgetInfo, pos2, vec2};

use super::paint_bevel;
use crate::theme::{self, SliderStyle};

/// A slider over a range of numbers, drawn in the theme's slider style.
///
/// It is changed by dragging, clicking, or with the arrow keys when it has
/// the keyboard focus (Shift moves ten times faster).
#[must_use = "add it with ui.add(...)"]
pub struct Slider<'a> {
    value: &'a mut f32,
    range: RangeInclusive<f32>,
    label: String,
    step: Option<f32>,
}

impl<'a> Slider<'a> {
    /// A slider for `value` in `range`, named `label` for screen readers.
    pub fn new(value: &'a mut f32, range: RangeInclusive<f32>, label: impl Into<String>) -> Self {
        Self { value, range, label: label.into(), step: None }
    }

    /// Snaps the value to multiples of `step`.
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }
}

impl Widget for Slider<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme::current(ui.ctx());
        let (lo, hi) = (*self.range.start(), *self.range.end());
        let height = t.metrics.control_height * 0.8;
        let width = ui.available_width().max(60.0);
        let (rect, mut response) = ui.allocate_exact_size(vec2(width, height), Sense::click_and_drag());
        let thumb_w = height * 0.7;
        let track = rect.shrink2(vec2(thumb_w * 0.5, 0.0));

        let old = *self.value;
        if let Some(pos) = response.interact_pointer_pos() {
            let f = ((pos.x - track.left()) / track.width()).clamp(0.0, 1.0);
            *self.value = lo + (hi - lo) * f;
        }
        if response.has_focus() {
            let fine = (hi - lo) / 100.0;
            let delta = ui.input(|i| {
                let big = if i.modifiers.shift { 10.0 } else { 1.0 };
                (f32::from(u8::from(i.key_pressed(Key::ArrowRight)))
                    - f32::from(u8::from(i.key_pressed(Key::ArrowLeft))))
                    * fine
                    * big
            });
            *self.value += delta;
        }
        if let Some(step) = self.step.filter(|s| *s > 0.0) {
            *self.value = (*self.value / step).round() * step;
        }
        *self.value = self.value.clamp(lo, hi);
        if *self.value != old {
            response.mark_changed();
        }
        let value = *self.value;
        response.widget_info(|| WidgetInfo::slider(true, f64::from(value), &self.label));

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let f = if hi > lo { (value - lo) / (hi - lo) } else { 0.0 };
            let x = track.left() + track.width() * f;
            let accent = t.c(|p| p.accent);
            match t.components.slider {
                SliderStyle::Groove => {
                    let groove = egui::Rect::from_center_size(rect.center(), vec2(rect.width(), height * 0.36));
                    let r = CornerRadius::same((groove.height() * 0.5) as u8);
                    painter.rect_filled(groove, r, t.c(|p| p.sunken));
                    painter.rect_filled(
                        egui::Rect::from_min_max(groove.min, pos2(x, groove.max.y)),
                        r,
                        accent.gamma_multiply(0.85),
                    );
                    let thumb = egui::Rect::from_center_size(pos2(x, rect.center().y), vec2(thumb_w, height));
                    let fill = if response.hovered() || response.dragged() {
                        t.c(|p| p.raised).gamma_multiply(1.15)
                    } else {
                        t.c(|p| p.raised)
                    };
                    paint_bevel(
                        painter,
                        thumb,
                        &t,
                        fill,
                        CornerRadius::same(t.metrics.radius.min(4.0) as u8),
                        response.dragged(),
                    );
                }
                SliderStyle::Line => {
                    let y = rect.center().y;
                    painter
                        .line_segment([pos2(rect.left(), y), pos2(rect.right(), y)], Stroke::new(2.0, t.c(|p| p.line)));
                    painter.line_segment([pos2(rect.left(), y), pos2(x, y)], Stroke::new(2.0, accent));
                    painter.circle_filled(pos2(x, y), height * 0.32, accent);
                }
            }
            if response.has_focus() {
                painter.rect_stroke(
                    rect.expand(1.0),
                    CornerRadius::same(3),
                    Stroke::new(1.5, accent),
                    egui::StrokeKind::Outside,
                );
            }
        }
        response
    }
}
