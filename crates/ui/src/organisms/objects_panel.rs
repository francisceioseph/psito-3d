//! The objects: add, pick, and change one.

use egui::Ui;
use psito3d_scene::{Combine, Material, ObjectId, Scene, Shape};

use crate::I18n;
use crate::atoms::{Button, Icon, IconButton, Swatch, caption};
use crate::molecules::{ChoiceChips, LabeledSlider, section};

fn shape_icon(shape: Shape) -> Icon {
    match shape {
        Shape::Sphere => Icon::Sphere,
        Shape::Cube => Icon::Cube,
        Shape::Cylinder => Icon::Cylinder,
        Shape::Cone => Icon::Cone,
        Shape::Torus => Icon::Torus,
    }
}

fn shape_key(shape: Shape) -> &'static str {
    match shape {
        Shape::Sphere => "sphere",
        Shape::Cube => "cube",
        Shape::Cylinder => "cylinder",
        Shape::Cone => "cone",
        Shape::Torus => "torus",
    }
}

/// Makes a preset material.
type MakeMaterial = fn() -> Material;

/// A material preset and the message naming it.
const PRESETS: [(&str, MakeMaterial); 4] = [
    ("material-chrome", Material::chrome),
    ("material-gold", Material::gold),
    ("material-glass", Material::glass),
    ("material-clay", || Material::clay([0.85, 0.35, 0.25])),
];

/// The objects section. `selected` is the object being edited.
pub fn objects_panel(ui: &mut Ui, i18n: &I18n, scene: &mut Scene, selected: &mut Option<ObjectId>) {
    section(ui, &i18n.tr("section-objects"), |ui| {
        ui.horizontal_wrapped(|ui| {
            for shape in Shape::ALL {
                let label = i18n.tr(&format!("add-{}", shape_key(shape)));
                if ui.add(IconButton::new(shape_icon(shape), label)).clicked() {
                    let material =
                        if shape == Shape::Sphere { Material::chrome() } else { Material::clay([0.85, 0.35, 0.25]) };
                    if let Some(id) = scene.add_object(shape, material) {
                        if let Some(o) = scene.object_mut(id) {
                            o.name = format!("{} {}", i18n.tr(&format!("shape-{}", shape_key(shape))), id.0);
                        }
                        *selected = Some(id);
                    }
                }
            }
        });

        if scene.objects.is_empty() {
            caption(ui, i18n.tr("no-objects"));
            return;
        }
        for o in &mut scene.objects {
            ui.horizontal(|ui| {
                let (icon, label) = if o.visible {
                    (Icon::Eye, i18n.tr("hide-object"))
                } else {
                    (Icon::EyeOff, i18n.tr("show-object"))
                };
                if ui.add(IconButton::new(icon, label)).clicked() {
                    o.visible = !o.visible;
                }
                let is_selected = *selected == Some(o.id);
                if ui.add(Button::new(&o.name).icon(shape_icon(o.shape)).selected(is_selected)).clicked() {
                    *selected = if is_selected { None } else { Some(o.id) };
                }
            });
        }

        let Some(id) = *selected else { return };
        let extent = scene.terrain.extent;
        let height = scene.terrain.height;
        let mut delete = false;
        if let Some(o) = scene.object_mut(id) {
            ui.separator();
            ui.add(egui::TextEdit::singleline(&mut o.name).hint_text(i18n.tr("object-name")))
                .widget_info(|| egui::WidgetInfo::labeled(egui::Role::TextInput, true, i18n.tr("object-name")));
            caption(ui, i18n.tr("material"));
            ui.horizontal_wrapped(|ui| {
                for (key, make) in PRESETS {
                    if ui.add(Button::new(i18n.tr(key))).clicked() {
                        o.material = make();
                    }
                }
            });
            ui.horizontal(|ui| {
                caption(ui, i18n.tr("color"));
                ui.add(Swatch::new(&mut o.material.color, i18n.tr("color")));
            });
            ui.add(LabeledSlider::new(&mut o.material.roughness, 0.0..=1.0, i18n.tr("roughness")));
            ui.add(LabeledSlider::new(&mut o.material.metallic, 0.0..=1.0, i18n.tr("metallic")));
            ui.add(LabeledSlider::new(&mut o.material.transmission, 0.0..=1.0, i18n.tr("transparency")));

            caption(ui, i18n.tr("combine"));
            let options = Combine::ALL
                .into_iter()
                .map(|c| {
                    let key = match c {
                        Combine::Union => "combine-union",
                        Combine::Subtract => "combine-subtract",
                        Combine::Intersect => "combine-intersect",
                    };
                    (c, i18n.tr(key))
                })
                .collect();
            ChoiceChips::new(&mut o.combine, options).show(ui);
            ui.add(LabeledSlider::new(&mut o.blend, 0.0..=10.0, i18n.tr("blend")).format(|v| format!("{v:.1}")));

            let p = &mut o.transform.position;
            let half = extent * 0.75;
            ui.add(LabeledSlider::new(&mut p.x, -half..=half, i18n.tr("position-x")).format(|v| format!("{v:.1}")));
            ui.add(
                LabeledSlider::new(&mut p.y, -height..=height * 4.0, i18n.tr("position-y"))
                    .format(|v| format!("{v:.1}")),
            );
            ui.add(LabeledSlider::new(&mut p.z, -half..=half, i18n.tr("position-z")).format(|v| format!("{v:.1}")));
            let mut size = o.transform.scale.max_element();
            if ui
                .add(LabeledSlider::new(&mut size, 0.5..=60.0, i18n.tr("size")).format(|v| format!("{v:.1}")))
                .changed()
            {
                let old = o.transform.scale.max_element().max(1e-3);
                o.transform.scale *= size / old;
            }
            let r = &mut o.transform.rotation;
            ui.add(LabeledSlider::new(&mut r.y, -180.0..=180.0, i18n.tr("turn")).format(|v| format!("{v:.0}°")));
            ui.add(LabeledSlider::new(&mut r.x, -180.0..=180.0, i18n.tr("tilt")).format(|v| format!("{v:.0}°")));
            delete = ui.add(Button::new(i18n.tr("delete-object")).icon(Icon::Trash)).clicked();
        } else {
            *selected = None;
        }
        if delete {
            scene.remove_object(id);
            *selected = None;
        }
    });
}
