//! The seed of a random world, and the die that rolls a new one.

use egui::{DragValue, Ui};

use crate::atoms::{Icon, IconButton, caption};

/// What happened in a [`SeedField`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SeedFieldResponse {
    /// The person typed or dragged a new seed.
    pub changed: bool,
    /// The person rolled the die; the caller picks the new seed.
    pub rolled: bool,
}

/// A seed number with a die beside it.
pub struct SeedField<'a> {
    seed: &'a mut u64,
    label: String,
    roll_label: String,
}

impl<'a> SeedField<'a> {
    /// A field for `seed`, labelled `label`; `roll_label` names the die.
    pub fn new(seed: &'a mut u64, label: impl Into<String>, roll_label: impl Into<String>) -> Self {
        Self { seed, label: label.into(), roll_label: roll_label.into() }
    }

    /// Shows the field.
    pub fn show(self, ui: &mut Ui) -> SeedFieldResponse {
        ui.horizontal(|ui| {
            caption(ui, &self.label);
            let drag = ui.add(DragValue::new(self.seed).range(0..=u32::MAX as u64));
            drag.widget_info(|| egui::WidgetInfo::labeled(egui::Role::SpinButton, true, &self.label));
            let rolled = ui.add(IconButton::new(Icon::Dice, self.roll_label)).clicked();
            SeedFieldResponse { changed: drag.changed(), rolled }
        })
        .inner
    }
}
