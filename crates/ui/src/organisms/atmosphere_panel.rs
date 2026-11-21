//! The sea and the fog.

use egui::Ui;
use psito3d_scene::{Fog, Sea};

use crate::I18n;
use crate::atoms::{Swatch, Toggle, caption};
use crate::molecules::{LabeledSlider, section};

/// The sea section and the fog section.
pub fn atmosphere_panel(ui: &mut Ui, i18n: &I18n, sea: &mut Sea, fog: &mut Fog) {
    section(ui, &i18n.tr("section-sea"), |ui| {
        ui.horizontal(|ui| {
            caption(ui, i18n.tr("sea"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(Toggle::new(&mut sea.enabled, i18n.tr("sea")));
                ui.add(Swatch::new(&mut sea.color, i18n.tr("sea-color")));
            });
        });
        ui.add_enabled_ui(sea.enabled, |ui| {
            ui.add(LabeledSlider::new(&mut sea.level, 0.0..=1.0, i18n.tr("sea-level")));
            ui.add(LabeledSlider::new(&mut sea.ripple, 0.0..=1.0, i18n.tr("ripple")));
        });
    });
    ui.add_space(6.0);
    section(ui, &i18n.tr("section-fog"), |ui| {
        ui.add(LabeledSlider::new(&mut fog.density, 0.0..=1.0, i18n.tr("fog")));
        ui.add(LabeledSlider::new(&mut fog.mist, 0.0..=1.0, i18n.tr("mist")));
        ui.add(LabeledSlider::new(&mut fog.mist_height, 0.05..=1.0, i18n.tr("mist-height")));
    });
}
