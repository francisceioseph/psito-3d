//! A row of buttons where one is chosen.

use egui::{Response, Ui};

use crate::atoms::Button;

/// Buttons side by side, one of them selected: a segmented control for a
/// small set of options. Wraps to more lines when narrow.
pub struct ChoiceChips<'a, T> {
    current: &'a mut T,
    options: Vec<(T, String)>,
}

impl<'a, T: PartialEq + Clone> ChoiceChips<'a, T> {
    /// Chips for `options` (value and label), changing `current`.
    pub fn new(current: &'a mut T, options: Vec<(T, String)>) -> Self {
        Self { current, options }
    }

    /// Shows the chips. The response is changed when the choice changed.
    pub fn show(self, ui: &mut Ui) -> Response {
        let mut changed = false;
        let mut response = ui
            .horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                for (value, label) in self.options {
                    let selected = *self.current == value;
                    if ui.add(Button::new(label).selected(selected)).clicked() && !selected {
                        *self.current = value;
                        changed = true;
                    }
                }
            })
            .response;
        if changed {
            response.mark_changed();
        }
        response
    }
}
