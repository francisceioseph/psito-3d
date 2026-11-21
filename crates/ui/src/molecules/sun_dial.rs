//! The sun dial: drag the sun around the sky.

use egui::{Color32, Key, Pos2, Response, Sense, Stroke, Ui, Widget, WidgetInfo, vec2};
use psito3d_scene::Sun;

use crate::theme;

/// The sky seen from below, as a disc: the middle is straight up, the rim is
/// the horizon, and the dark ring outside is below it (night). Dragging the
/// sun moves it; the arrow keys turn it (left, right) and raise it (up, down).
#[must_use = "add it with ui.add(...)"]
pub struct SunDial<'a> {
    sun: &'a mut Sun,
    label: String,
    size: f32,
}

/// How far below the horizon the outer ring reaches, in degrees.
const NIGHT_DEPTH: f32 = 30.0;

impl<'a> SunDial<'a> {
    /// A dial for `sun`, named `label`.
    pub fn new(sun: &'a mut Sun, label: impl Into<String>) -> Self {
        Self { sun, label: label.into(), size: 150.0 }
    }

    /// The diameter, in points.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

/// Where on the dial the sun is drawn, as an offset from the centre in
/// units of the horizon radius.
pub(crate) fn sun_to_offset(sun: &Sun) -> egui::Vec2 {
    let r = (90.0 - sun.elevation.clamp(-NIGHT_DEPTH, 90.0)) / 90.0;
    let az = sun.azimuth.to_radians();
    vec2(az.sin(), -az.cos()) * r
}

/// The sun for an offset from the centre (the inverse of [`sun_to_offset`]).
pub(crate) fn offset_to_sun(offset: egui::Vec2) -> Sun {
    let r = offset.length().min((90.0 + NIGHT_DEPTH) / 90.0);
    let azimuth = offset.x.atan2(-offset.y).to_degrees().rem_euclid(360.0);
    Sun { azimuth, elevation: 90.0 - r * 90.0 }
}

impl Widget for SunDial<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme::current(ui.ctx());
        let (rect, mut response) = ui.allocate_exact_size(vec2(self.size, self.size), Sense::click_and_drag());
        let center = rect.center();
        let outer = self.size * 0.5;
        let horizon = outer * 90.0 / (90.0 + NIGHT_DEPTH);

        let before = *self.sun;
        if let Some(pos) = response.interact_pointer_pos() {
            *self.sun = offset_to_sun((pos - center) / horizon);
        }
        if response.has_focus() {
            ui.input(|i| {
                let step = if i.modifiers.shift { 10.0 } else { 2.0 };
                if i.key_pressed(Key::ArrowLeft) {
                    self.sun.azimuth = (self.sun.azimuth - step).rem_euclid(360.0);
                }
                if i.key_pressed(Key::ArrowRight) {
                    self.sun.azimuth = (self.sun.azimuth + step).rem_euclid(360.0);
                }
                if i.key_pressed(Key::ArrowUp) {
                    self.sun.elevation = (self.sun.elevation + step).min(90.0);
                }
                if i.key_pressed(Key::ArrowDown) {
                    self.sun.elevation = (self.sun.elevation - step).max(-NIGHT_DEPTH);
                }
            });
        }
        if *self.sun != before {
            response.mark_changed();
        }
        let sun = *self.sun;
        response.widget_info(|| WidgetInfo::slider(true, f64::from(sun.elevation), &self.label));

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            // Night ring, then the sky getting paler towards the horizon.
            painter.circle_filled(center, outer, t.c(|p| p.sunken));
            let day = ((sun.elevation + 6.0) / 30.0).clamp(0.0, 1.0);
            let zenith = Color32::from_rgb(40, 90, 170).lerp_to_gamma(Color32::from_rgb(10, 14, 40), 1.0 - day);
            let rim = Color32::from_rgb(170, 200, 235).lerp_to_gamma(Color32::from_rgb(40, 40, 80), 1.0 - day);
            for i in (1..=12).rev() {
                let f = i as f32 / 12.0;
                painter.circle_filled(center, horizon * f, zenith.lerp_to_gamma(rim, f));
            }
            painter.circle_stroke(center, horizon, Stroke::new(t.metrics.stroke, t.c(|p| p.line)));
            // Compass ticks.
            for k in 0..4 {
                let a = k as f32 * std::f32::consts::FRAC_PI_2;
                let d = vec2(a.sin(), -a.cos());
                painter.line_segment(
                    [center + d * horizon * 0.92, center + d * horizon],
                    Stroke::new(1.0, t.c(|p| p.text_muted)),
                );
            }
            // The sun, with a soft glow.
            let pos: Pos2 = center + sun_to_offset(&sun) * horizon;
            let accent = t.c(|p| p.accent);
            for (r, a) in [(16.0, 0.12), (11.0, 0.25)] {
                painter.circle_filled(pos, r, accent.gamma_multiply(a));
            }
            painter.circle_filled(pos, 7.0, accent);
            if response.has_focus() {
                painter.circle_stroke(center, outer + 2.0, Stroke::new(1.5, accent));
            }
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets_round_trip() {
        for (az, el) in [(0.0, 45.0), (90.0, 10.0), (200.0, 70.0), (300.0, -20.0)] {
            let back = offset_to_sun(sun_to_offset(&Sun { azimuth: az, elevation: el }));
            assert!((back.azimuth - az).abs() < 1e-3 && (back.elevation - el).abs() < 1e-3, "{az},{el} -> {back:?}");
        }
    }

    #[test]
    fn the_middle_is_noon_high_and_the_rim_is_the_horizon() {
        assert!((offset_to_sun(vec2(0.0, 0.0)).elevation - 90.0).abs() < 1e-4);
        assert!(offset_to_sun(vec2(1.0, 0.0)).elevation.abs() < 1e-4);
        assert_eq!(offset_to_sun(vec2(5.0, 0.0)).elevation, -NIGHT_DEPTH);
    }
}
