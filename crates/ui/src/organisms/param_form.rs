//! The controls of a generator's parameters, built from their descriptions.

use egui::Ui;
use psito3d_scene::{ParamKind, ParamSpec, ParamValue, ParamValues};

use crate::atoms::{Toggle, caption};
use crate::molecules::{ChoiceChips, LabeledSlider};

/// One control per parameter in `specs`, editing `values` (which should
/// already be resolved, so every parameter has a value). Labels are shown
/// in `lang`. Returns whether a value changed.
pub fn param_form(ui: &mut Ui, lang: &str, specs: &[ParamSpec], values: &mut ParamValues) -> bool {
    let mut changed = false;
    for spec in specs {
        let label = spec.label.get(lang).to_owned();
        let value = values.entry(spec.id.clone()).or_insert_with(|| spec.default_value());
        match &spec.kind {
            ParamKind::Slider { min, max, .. } => {
                let mut v = value.as_f32(*min);
                if ui.add(LabeledSlider::new(&mut v, *min..=*max, label)).changed() {
                    *value = ParamValue::Number(v);
                    changed = true;
                }
            }
            ParamKind::Toggle { .. } => {
                let mut on = value.as_bool(false);
                ui.horizontal(|ui| {
                    caption(ui, &label);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(Toggle::new(&mut on, &label)).changed() {
                            *value = ParamValue::Bool(on);
                            changed = true;
                        }
                    });
                });
            }
            ParamKind::Choice { options, default } => {
                caption(ui, &label);
                let mut current = match value {
                    ParamValue::Choice(id) => id.clone(),
                    _ => default.clone(),
                };
                let opts = options.iter().map(|o| (o.id.clone(), o.label.get(lang).to_owned())).collect();
                if ChoiceChips::new(&mut current, opts).show(ui).changed() {
                    *value = ParamValue::Choice(current);
                    changed = true;
                }
            }
        }
    }
    changed
}
