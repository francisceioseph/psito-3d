//! A slider with its label and value.

use std::ops::RangeInclusive;

use egui::{Response, Ui, Widget};

use crate::atoms::{Slider, caption, value_text};

/// A label and the value on one line, the slider under them.
#[must_use = "add it with ui.add(...)"]
pub struct LabeledSlider<'a> {
    value: &'a mut f32,
    range: RangeInclusive<f32>,
    label: String,
    format: Box<dyn Fn(f32) -> String + 'a>,
    step: Option<f32>,
}

impl<'a> LabeledSlider<'a> {
    /// A slider for `value` in `range`, labelled `label`, showing the value
    /// with two decimals.
    pub fn new(value: &'a mut f32, range: RangeInclusive<f32>, label: impl Into<String>) -> Self {
        Self { value, range, label: label.into(), format: Box::new(|v| format!("{v:.2}")), step: None }
    }

    /// Shows the value with `format`.
    pub fn format(mut self, format: impl Fn(f32) -> String + 'a) -> Self {
        self.format = Box::new(format);
        self
    }

    /// Snaps the value to multiples of `step`.
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }
}

impl Widget for LabeledSlider<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.horizontal(|ui| {
                caption(ui, &self.label);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    value_text(ui, (self.format)(*self.value));
                });
            });
            let mut slider = Slider::new(self.value, self.range, self.label);
            if let Some(s) = self.step {
                slider = slider.step(s);
            }
            ui.add(slider)
        })
        .inner
    }
}
