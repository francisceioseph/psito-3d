//! Vector icons, drawn with lines and shapes.
//!
//! The icons are painted instead of taken from a font, so they need no icon
//! font, scale to any size and follow the theme colours.

use egui::{Color32, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Widget, WidgetInfo, pos2, vec2};

use super::{control_radius, paint_bevel};
use crate::theme;

/// The icons of the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    /// A die, for "Surprise me".
    Dice,
    /// The sun.
    Sun,
    /// An arrow turning back.
    Undo,
    /// An arrow turning forward.
    Redo,
    /// A plus.
    Plus,
    /// A bin.
    Trash,
    /// An open folder.
    Open,
    /// A floppy disk.
    Save,
    /// A picture.
    Picture,
    /// An eye: shown.
    Eye,
    /// A crossed eye: hidden.
    EyeOff,
    /// A ball.
    Sphere,
    /// A box.
    Cube,
    /// A cylinder.
    Cylinder,
    /// A cone.
    Cone,
    /// A ring.
    Torus,
    /// A blank page.
    New,
    /// A screen on its stand: the desktop.
    Monitor,
    /// Three linked dots: share.
    Share,
    /// An arrow going round: turn around by itself.
    Orbit,
    /// A brush.
    Brush,
}

fn arc(center: Pos2, radius: f32, from: f32, to: f32) -> Vec<Pos2> {
    (0..=16)
        .map(|i| {
            let a = from + (to - from) * i as f32 / 16.0;
            center + vec2(a.cos(), a.sin()) * radius
        })
        .collect()
}

/// Paints `icon` filling `rect`, in `color`.
pub fn paint_icon(painter: &Painter, rect: Rect, icon: Icon, color: Color32) {
    let s = rect.width().min(rect.height());
    let c = rect.center();
    let w = (s * 0.11).max(1.2);
    let stroke = Stroke::new(w, color);
    let p = |x: f32, y: f32| c + vec2(x, y) * s * 0.5;
    let line = |pts: Vec<Pos2>| painter.add(Shape::line(pts, stroke));
    use std::f32::consts::PI;
    match icon {
        Icon::Dice => {
            painter.rect_stroke(
                Rect::from_center_size(c, vec2(s * 0.8, s * 0.8)),
                (s * 0.15) as u8,
                stroke,
                egui::StrokeKind::Middle,
            );
            for (x, y) in [(-0.4, -0.4), (0.4, -0.4), (0.0, 0.0), (-0.4, 0.4), (0.4, 0.4)] {
                painter.circle_filled(p(x * 0.8, y * 0.8), w * 0.9, color);
            }
        }
        Icon::Sun => {
            painter.circle_filled(c, s * 0.2, color);
            for i in 0..8 {
                let a = i as f32 * PI / 4.0;
                let d = vec2(a.cos(), a.sin());
                painter.line_segment([c + d * s * 0.32, c + d * s * 0.46], stroke);
            }
        }
        Icon::Undo | Icon::Redo => {
            let flip = if icon == Icon::Undo { 1.0 } else { -1.0 };
            let pts: Vec<Pos2> = arc(c + vec2(0.0, s * 0.05), s * 0.32, -PI * 0.15, PI * 1.15)
                .into_iter()
                .map(|q| pos2(c.x + (q.x - c.x) * flip, q.y))
                .collect();
            let start = pts[pts.len() - 1];
            line(pts);
            let tip = vec2(-flip, 0.0) * s * 0.0;
            painter.add(Shape::convex_polygon(
                vec![
                    start + tip + vec2(-0.2 * flip, -0.2) * s,
                    start + vec2(0.2 * flip, -0.12) * s,
                    start + vec2(-0.05 * flip, 0.18) * s,
                ],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Plus => {
            painter.line_segment([p(-0.7, 0.0), p(0.7, 0.0)], stroke);
            painter.line_segment([p(0.0, -0.7), p(0.0, 0.7)], stroke);
        }
        Icon::Trash => {
            line(vec![p(-0.5, -0.45), p(-0.4, 0.75), p(0.4, 0.75), p(0.5, -0.45)]);
            painter.line_segment([p(-0.7, -0.5), p(0.7, -0.5)], stroke);
            painter.line_segment([p(-0.2, -0.75), p(0.2, -0.75)], stroke);
        }
        Icon::Open => {
            line(vec![p(-0.8, 0.6), p(-0.8, -0.6), p(-0.3, -0.6), p(-0.15, -0.4), p(0.6, -0.4), p(0.6, -0.1)]);
            line(vec![p(-0.8, 0.6), p(-0.5, -0.1), p(0.85, -0.1), p(0.55, 0.6), p(-0.8, 0.6)]);
        }
        Icon::Save => {
            line(vec![p(-0.7, -0.7), p(0.45, -0.7), p(0.7, -0.45), p(0.7, 0.7), p(-0.7, 0.7), p(-0.7, -0.7)]);
            painter.rect_filled(Rect::from_min_max(p(-0.4, -0.7), p(0.3, -0.3)), 0, color);
            painter.rect_stroke(Rect::from_min_max(p(-0.45, 0.15), p(0.45, 0.7)), 0, stroke, egui::StrokeKind::Middle);
        }
        Icon::Picture => {
            painter.rect_stroke(
                Rect::from_center_size(c, vec2(s * 0.8, s * 0.64)),
                1,
                stroke,
                egui::StrokeKind::Middle,
            );
            line(vec![p(-0.7, 0.5), p(-0.25, -0.05), p(0.05, 0.25), p(0.3, 0.0), p(0.7, 0.45)]);
            painter.circle_filled(p(0.35, -0.3), s * 0.08, color);
        }
        Icon::Eye | Icon::EyeOff => {
            let mut top = arc(c + vec2(0.0, s * 0.45), s * 0.6, -PI * 0.78, -PI * 0.22);
            let bottom = arc(c - vec2(0.0, s * 0.45), s * 0.6, PI * 0.22, PI * 0.78);
            top.extend(bottom);
            top.push(top[0]);
            line(top);
            painter.circle_filled(c, s * 0.14, color);
            if icon == Icon::EyeOff {
                painter.line_segment([p(-0.75, 0.75), p(0.75, -0.75)], stroke);
            }
        }
        Icon::Sphere => {
            painter.circle_stroke(c, s * 0.42, stroke);
            painter.circle_filled(p(-0.3, -0.3), s * 0.08, color);
        }
        Icon::Cube => {
            line(vec![
                p(0.0, -0.75),
                p(0.7, -0.4),
                p(0.7, 0.4),
                p(0.0, 0.75),
                p(-0.7, 0.4),
                p(-0.7, -0.4),
                p(0.0, -0.75),
            ]);
            line(vec![p(-0.7, -0.4), p(0.0, -0.05), p(0.7, -0.4)]);
            painter.line_segment([p(0.0, -0.05), p(0.0, 0.75)], stroke);
        }
        Icon::Cylinder => {
            line(
                arc(p(0.0, -0.5), s * 0.32, 0.0, 2.0 * PI)
                    .into_iter()
                    .map(|q| pos2(q.x, c.y - s * 0.25 + (q.y - (c.y - s * 0.25)) * 0.35))
                    .collect(),
            );
            line(
                arc(p(0.0, 0.5), s * 0.32, 0.0, PI)
                    .into_iter()
                    .map(|q| pos2(q.x, c.y + s * 0.25 + (q.y - (c.y + s * 0.25)) * 0.35))
                    .collect(),
            );
            painter.line_segment([p(-0.64, -0.5), p(-0.64, 0.5)], stroke);
            painter.line_segment([p(0.64, -0.5), p(0.64, 0.5)], stroke);
        }
        Icon::Cone => {
            line(vec![p(-0.65, 0.5), p(0.0, -0.75), p(0.65, 0.5)]);
            line(
                arc(p(0.0, 0.5), s * 0.325, 0.0, PI)
                    .into_iter()
                    .map(|q| pos2(q.x, c.y + s * 0.25 + (q.y - (c.y + s * 0.25)) * 0.35))
                    .collect(),
            );
        }
        Icon::Torus => {
            let squash = |q: Pos2| pos2(q.x, c.y + (q.y - c.y) * 0.55);
            line(arc(c, s * 0.45, 0.0, 2.0 * PI).into_iter().map(squash).collect());
            line(arc(c, s * 0.18, 0.0, 2.0 * PI).into_iter().map(squash).collect());
        }
        Icon::Monitor => {
            painter.rect_stroke(Rect::from_min_max(p(-0.8, -0.65), p(0.8, 0.4)), 1, stroke, egui::StrokeKind::Middle);
            // A little mountain on the screen: it is a picture on the desktop.
            line(vec![p(-0.55, 0.25), p(-0.15, -0.2), p(0.1, 0.05), p(0.3, -0.1), p(0.6, 0.25)]);
            painter.line_segment([p(0.0, 0.4), p(0.0, 0.7)], stroke);
            painter.line_segment([p(-0.35, 0.72), p(0.35, 0.72)], stroke);
        }
        Icon::Share => {
            let (a, b, c) = (p(0.5, -0.55), p(-0.5, 0.0), p(0.5, 0.55));
            painter.line_segment([a, b], stroke);
            painter.line_segment([b, c], stroke);
            for q in [a, b, c] {
                painter.circle_filled(q, s * 0.13, color);
            }
        }
        Icon::Orbit => {
            let squash = |q: Pos2| pos2(q.x, c.y + (q.y - c.y) * 0.45);
            let pts: Vec<Pos2> = arc(c, s * 0.42, 0.3, 2.0 * PI - 0.3).into_iter().map(squash).collect();
            let end = pts[pts.len() - 1];
            line(pts);
            painter.add(Shape::convex_polygon(
                vec![end + vec2(0.0, -0.16) * s, end + vec2(0.0, 0.16) * s, end + vec2(0.2, 0.02) * s],
                color,
                Stroke::NONE,
            ));
            painter.circle_filled(c, s * 0.12, color);
        }
        Icon::Brush => {
            painter.line_segment([p(0.7, -0.7), p(-0.05, 0.1)], Stroke::new(w * 1.4, color));
            painter.add(Shape::convex_polygon(
                vec![p(-0.05, 0.05), p(0.05, 0.2), p(-0.3, 0.75), p(-0.75, 0.7), p(-0.45, 0.25)],
                color,
                Stroke::NONE,
            ));
        }
        Icon::New => {
            line(vec![p(-0.55, -0.75), p(0.2, -0.75), p(0.55, -0.4), p(0.55, 0.75), p(-0.55, 0.75), p(-0.55, -0.75)]);
            line(vec![p(0.2, -0.75), p(0.2, -0.4), p(0.55, -0.4)]);
        }
    }
}

/// A square button with only an icon. The label is still required: it is
/// the tooltip and the name screen readers say.
#[must_use = "add it with ui.add(...)"]
pub struct IconButton {
    icon: Icon,
    label: String,
    enabled: bool,
    selected: bool,
}

impl IconButton {
    /// A button showing `icon`, named `label`.
    pub fn new(icon: Icon, label: impl Into<String>) -> Self {
        Self { icon, label: label.into(), enabled: true, selected: false }
    }

    /// Greys it out and ignores clicks.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Shows it pressed in.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl Widget for IconButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme::current(ui.ctx());
        let side = t.metrics.control_height;
        let sense = if self.enabled { Sense::click() } else { Sense::hover() };
        let (rect, response) = ui.allocate_exact_size(vec2(side, side), sense);
        response.widget_info(|| WidgetInfo::selected(egui::Role::Button, self.enabled, self.selected, &self.label));
        if ui.is_rect_visible(rect) {
            let radius = control_radius(&t, side);
            let mut fill = if self.selected { t.c(|p| p.accent) } else { t.c(|p| p.raised) };
            if response.hovered() && self.enabled {
                fill = fill.gamma_multiply(1.12);
            }
            paint_bevel(ui.painter(), rect, &t, fill, radius, response.is_pointer_button_down_on() || self.selected);
            let color = match (self.enabled, self.selected) {
                (false, _) => t.c(|p| p.text_muted),
                (true, true) => t.c(|p| p.accent_text),
                (true, false) => t.c(|p| p.text),
            };
            paint_icon(ui.painter(), rect.shrink(side * 0.22), self.icon, color);
        }
        response.on_hover_text(self.label)
    }
}
