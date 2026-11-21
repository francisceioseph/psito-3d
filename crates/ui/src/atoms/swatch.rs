//! A colour well that opens a picker.

use egui::{Response, Ui, Widget};

/// A colour well for a linear RGB colour, named `label`. It uses egui's
/// own picker, which the theme has restyled.
#[must_use = "add it with ui.add(...)"]
pub struct Swatch<'a> {
    rgb: &'a mut [f32; 3],
    label: String,
}

impl<'a> Swatch<'a> {
    /// A well for `rgb`.
    pub fn new(rgb: &'a mut [f32; 3], label: impl Into<String>) -> Self {
        Self { rgb, label: label.into() }
    }
}

impl Widget for Swatch<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut response = ui.color_edit_button_rgb(self.rgb);
        let label = self.label;
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::ColorWell, true, &label));
        response = response.on_hover_text(label);
        response
    }
}
