//! The terrain: where the heights come from and their shape.

use egui::Ui;
use psito3d_scene::{ParamSpec, ParamValues, TerrainSettings, TerrainSource, resolve_params};

use crate::I18n;
use crate::atoms::{Button, Icon, caption};
use crate::molecules::{ChoiceChips, LabeledSlider, SeedField, section};

/// A generator the person can pick, already described in their language.
#[derive(Debug, Clone, PartialEq)]
pub struct GeneratorChoice {
    /// The id stored in the scene.
    pub id: String,
    /// The name in the menu.
    pub name: String,
    /// One sentence on what it makes.
    pub description: String,
    /// Its parameters.
    pub params: Vec<ParamSpec>,
}

/// What the terrain panel asks the editor to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainAction {
    /// Open a picture to use as the terrain.
    PickPicture,
    /// Roll a new seed.
    RollSeed,
}

/// The terrain section.
pub fn terrain_panel(
    ui: &mut Ui,
    i18n: &I18n,
    terrain: &mut TerrainSettings,
    generators: &[GeneratorChoice],
) -> Option<TerrainAction> {
    let mut action = None;
    section(ui, &i18n.tr("section-terrain"), |ui| {
        let picture = i18n.tr("source-picture");
        let current_name = match &terrain.source {
            TerrainSource::Generator { id, .. } => {
                generators.iter().find(|g| &g.id == id).map_or(id.clone(), |g| g.name.clone())
            }
            TerrainSource::Image { .. } => picture.clone(),
        };
        let mut picked: Option<&GeneratorChoice> = None;
        let mut pick_picture = false;
        egui::ComboBox::from_label(i18n.tr("generator"))
            .selected_text(&current_name)
            .width(ui.available_width() * 0.6)
            .show_ui(ui, |ui| {
                for g in generators {
                    if ui.selectable_label(g.name == current_name, &g.name).on_hover_text(&g.description).clicked() {
                        picked = Some(g);
                    }
                }
                ui.separator();
                if ui.selectable_label(false, &picture).clicked() {
                    pick_picture = true;
                }
            });
        if let Some(g) = picked {
            // Choosing a generator is an edit: its defaults go into the scene.
            terrain.source =
                TerrainSource::Generator { id: g.id.clone(), params: resolve_params(&g.params, &ParamValues::new()) };
        }
        if pick_picture {
            action = Some(TerrainAction::PickPicture);
        }

        match &mut terrain.source {
            TerrainSource::Generator { id, params } => {
                if let Some(g) = generators.iter().find(|g| &g.id == id) {
                    if !g.description.is_empty() {
                        caption(ui, &g.description);
                    }
                    // Showing the form never changes the scene; only moving a
                    // control does, so opening a scene is not an undo step.
                    let mut values = resolve_params(&g.params, params);
                    if super::param_form(ui, i18n.lang(), &g.params, &mut values) {
                        *params = values;
                    }
                } else {
                    caption(ui, i18n.tr("generator-missing"));
                }
                let seed = SeedField::new(&mut terrain.seed, i18n.tr("seed"), i18n.tr("roll-seed")).show(ui);
                if seed.rolled {
                    action = Some(TerrainAction::RollSeed);
                }
            }
            TerrainSource::Image { path } => {
                let name =
                    path.file_name().map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned());
                caption(ui, name);
                if ui.add(Button::new(i18n.tr("choose-picture")).icon(Icon::Picture)).clicked() {
                    action = Some(TerrainAction::PickPicture);
                }
            }
        }

        ui.add(
            LabeledSlider::new(&mut terrain.height, 2.0..=80.0, i18n.tr("terrain-height"))
                .format(|v| format!("{v:.0}")),
        );
        ui.add(LabeledSlider::new(&mut terrain.edge_falloff, 0.0..=0.5, i18n.tr("coast")));
        caption(ui, i18n.tr("resolution"));
        ChoiceChips::new(
            &mut terrain.resolution,
            [256u32, 512, 1024].into_iter().map(|r| (r, r.to_string())).collect(),
        )
        .show(ui);
    });
    action
}
