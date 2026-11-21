//! A titled section of the side panel.

use egui::Ui;

use crate::atoms::heading;
use crate::theme;

/// A card with a heading, for one group of settings.
pub fn section<R>(ui: &mut Ui, title: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    let t = theme::current(ui.ctx());
    egui::Frame::new()
        .fill(t.c(|p| p.background).gamma_multiply(0.6))
        .stroke(egui::Stroke::new(t.metrics.stroke, t.c(|p| p.line)))
        .corner_radius(t.metrics.radius as u8)
        .inner_margin(t.metrics.spacing)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            heading(ui, title);
            ui.add_space(t.metrics.spacing * 0.5);
            add_contents(ui)
        })
        .inner
}
