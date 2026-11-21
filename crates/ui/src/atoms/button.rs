//! A push button.

use egui::{Align2, FontId, Response, Sense, Ui, Widget, WidgetInfo};

use super::{Icon, control_radius, paint_bevel, paint_icon};
use crate::theme::{self, ButtonStyle};

/// How much a button stands out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonKind {
    /// An ordinary button.
    #[default]
    Normal,
    /// The main action of its area, in the accent colour.
    Primary,
}

/// A push button with a label and an optional icon, drawn in the theme's
/// button style (bevel, flat or pill).
#[must_use = "add it with ui.add(...)"]
pub struct Button {
    label: String,
    icon: Option<Icon>,
    kind: ButtonKind,
    enabled: bool,
    selected: bool,
}

impl Button {
    /// A button reading `label`.
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), icon: None, kind: ButtonKind::Normal, enabled: true, selected: false }
    }

    /// Draws `icon` before the label.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Makes it the primary button.
    pub fn primary(mut self) -> Self {
        self.kind = ButtonKind::Primary;
        self
    }

    /// Greys it out and ignores clicks.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Shows it pressed in, for a choice that is on.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl Widget for Button {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme::current(ui.ctx());
        let font = FontId::proportional(t.typography.body);
        let galley = ui.painter().layout_no_wrap(self.label.clone(), font.clone(), t.c(|p| p.text));
        let icon_w = if self.icon.is_some() { t.typography.body + 6.0 } else { 0.0 };
        let pad = t.metrics.spacing * 1.25;
        let size = egui::vec2(galley.size().x + icon_w + pad * 2.0, t.metrics.control_height);
        let sense = if self.enabled { Sense::click() } else { Sense::hover() };
        let (rect, response) = ui.allocate_exact_size(size, sense);
        response.widget_info(|| WidgetInfo::selected(egui::Role::Button, self.enabled, self.selected, &self.label));

        if ui.is_rect_visible(rect) {
            let primary = self.kind == ButtonKind::Primary || self.selected;
            let mut fill = if primary { t.c(|p| p.accent) } else { t.c(|p| p.raised) };
            if response.hovered() && self.enabled {
                fill = fill.gamma_multiply(1.12);
            }
            let text_color = if !self.enabled {
                t.c(|p| p.text_muted)
            } else if primary {
                t.c(|p| p.accent_text)
            } else {
                t.c(|p| p.text)
            };
            let radius = control_radius(&t, rect.height());
            let pressed = response.is_pointer_button_down_on() || self.selected;
            match t.components.button {
                ButtonStyle::Bevel => paint_bevel(ui.painter(), rect, &t, fill, radius, pressed),
                ButtonStyle::Flat | ButtonStyle::Pill => {
                    ui.painter().rect_filled(rect, radius, fill);
                }
            }
            if response.has_focus() {
                ui.painter().rect_stroke(
                    rect.expand(2.0),
                    radius,
                    egui::Stroke::new(1.5, t.c(|p| p.accent)),
                    egui::StrokeKind::Outside,
                );
            }
            let nudge = if pressed && t.components.button == ButtonStyle::Bevel {
                egui::vec2(0.0, 1.0)
            } else {
                egui::Vec2::ZERO
            };
            let mut x = rect.left() + pad;
            if let Some(icon) = self.icon {
                let s = t.typography.body;
                let icon_rect =
                    egui::Rect::from_center_size(egui::pos2(x + s * 0.5, rect.center().y) + nudge, egui::vec2(s, s));
                paint_icon(ui.painter(), icon_rect, icon, text_color);
                x += icon_w;
            }
            ui.painter().text(
                egui::pos2(x, rect.center().y) + nudge,
                Align2::LEFT_CENTER,
                &self.label,
                font,
                text_color,
            );
        }
        response
    }
}
