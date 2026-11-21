//! The sky: the sun, the haze, the clouds and the exposure.

use egui::Ui;
use psito3d_scene::Sky;

use crate::I18n;
use crate::molecules::{LabeledSlider, SunDial, section};

/// The sky section.
pub fn sky_panel(ui: &mut Ui, i18n: &I18n, sky: &mut Sky) {
    section(ui, &i18n.tr("section-sky"), |ui| {
        ui.vertical_centered(|ui| {
            ui.add(SunDial::new(&mut sky.sun, i18n.tr("sun")).size(150.0));
        });
        let deg = |v: f32| format!("{v:.0}°");
        ui.add(LabeledSlider::new(&mut sky.sun.elevation, -30.0..=90.0, i18n.tr("sun-height")).format(deg));
        ui.add(LabeledSlider::new(&mut sky.sun.azimuth, 0.0..=360.0, i18n.tr("sun-direction")).format(deg));
        ui.add(LabeledSlider::new(&mut sky.haze, 0.0..=1.0, i18n.tr("haze")));
        ui.add(LabeledSlider::new(&mut sky.cloud_cover, 0.0..=1.0, i18n.tr("clouds")));
        ui.add(LabeledSlider::new(&mut sky.cloud_height, 1.0..=12.0, i18n.tr("cloud-height")));
        ui.add(LabeledSlider::new(&mut sky.exposure, -3.0..=3.0, i18n.tr("exposure")).format(|v| format!("{v:+.1}")));
    });
}
