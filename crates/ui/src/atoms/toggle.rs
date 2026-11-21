//! An on/off switch.

use egui::{Response, Sense, Ui, Widget, WidgetInfo, pos2, vec2};

use crate::theme;

/// A switch for a flag, named `label` for screen readers.
#[must_use = "add it with ui.add(...)"]
pub struct Toggle<'a> {
    on: &'a mut bool,
    label: String,
}

impl<'a> Toggle<'a> {
    /// A switch for `on`.
    pub fn new(on: &'a mut bool, label: impl Into<String>) -> Self {
        Self { on, label: label.into() }
    }
}

impl Widget for Toggle<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = theme::current(ui.ctx());
        let h = t.metrics.control_height * 0.7;
        let (rect, mut response) = ui.allocate_exact_size(vec2(h * 1.8, h), Sense::click());
        if response.clicked() {
            *self.on = !*self.on;
            response.mark_changed();
        }
        let on = *self.on;
        response.widget_info(|| WidgetInfo::selected(egui::Role::Switch, true, on, &self.label));
        if ui.is_rect_visible(rect) {
            let how_on = ui.ctx().animate_bool_responsive(response.id, on);
            let fill = if on { t.c(|p| p.accent) } else { t.c(|p| p.sunken) };
            let r = egui::CornerRadius::same((h * 0.5) as u8);
            ui.painter().rect_filled(rect, r, fill);
            let x = egui::lerp(rect.left() + h * 0.5..=rect.right() - h * 0.5, how_on);
            ui.painter().circle_filled(pos2(x, rect.center().y), h * 0.38, t.c(|p| p.raised));
            if response.has_focus() {
                ui.painter().rect_stroke(
                    rect.expand(2.0),
                    r,
                    egui::Stroke::new(1.5, t.c(|p| p.accent)),
                    egui::StrokeKind::Outside,
                );
            }
        }
        response
    }
}
