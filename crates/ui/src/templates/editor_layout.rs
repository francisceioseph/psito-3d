//! The editor's layout: toolbar on top, settings on the right, status at
//! the bottom, and the image filling the rest.

use egui::{Frame, Panel, Ui};

use crate::theme;

/// The editor's layout. Each part is a closure, so the page decides what
/// goes in it and the template only decides where.
pub struct EditorLayout {
    /// The width of the settings panel, in points.
    pub sidebar_width: f32,
}

impl Default for EditorLayout {
    fn default() -> Self {
        Self { sidebar_width: 310.0 }
    }
}

impl EditorLayout {
    /// Lays out the four parts in `ui`.
    pub fn show(
        &self,
        ui: &mut Ui,
        toolbar: impl FnOnce(&mut Ui),
        sidebar: impl FnOnce(&mut Ui),
        status: impl FnOnce(&mut Ui),
        viewport: impl FnOnce(&mut Ui),
    ) {
        let t = theme::current(ui.ctx());
        let s = t.metrics.spacing;
        let bar = Frame::new().fill(t.c(|p| p.panel)).inner_margin(egui::Margin::symmetric(s as i8, (s * 0.75) as i8));
        Panel::top("toolbar").frame(bar).show(ui, toolbar);
        Panel::bottom("status").frame(bar).show(ui, status);
        Panel::right("sidebar")
            .frame(Frame::new().fill(t.c(|p| p.panel)).inner_margin(s as i8))
            .default_size(self.sidebar_width)
            .size_range(240.0..=520.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = s * 0.75;
                    sidebar(ui);
                });
            });
        egui::CentralPanel::no_frame().show(ui, viewport);
    }
}
