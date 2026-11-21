//! The bar along the top.

use egui::Ui;
use psito3d_scene::Filter;

use crate::I18n;
use crate::atoms::{Button, Icon, IconButton};
use crate::i18n::LANGUAGES;

/// What the toolbar asks the editor to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolbarAction {
    /// A whole new random world.
    Surprise,
    /// Start from the starter scene.
    New,
    /// Open a scene file.
    Open,
    /// Save the scene.
    Save,
    /// Render and save a picture.
    Export,
    /// Undo the last change.
    Undo,
    /// Redo it.
    Redo,
    /// Switch to the theme with this id.
    Theme(String),
    /// Switch to the language with this code.
    Language(String),
}

/// The toolbar and what it needs to know.
pub struct Toolbar<'a> {
    /// Whether there is something to undo.
    pub can_undo: bool,
    /// Whether there is something to redo.
    pub can_redo: bool,
    /// The themes on offer: id and name.
    pub themes: &'a [(String, String)],
    /// The id of the theme in effect.
    pub theme: &'a str,
    /// The finishing filter, edited in place.
    pub filter: &'a mut Filter,
}

impl Toolbar<'_> {
    /// Shows the toolbar; returns the action clicked, if any.
    pub fn show(self, ui: &mut Ui, i18n: &I18n) -> Option<ToolbarAction> {
        let mut action = None;
        let mut act = |a: ToolbarAction| action = Some(a);
        ui.horizontal_wrapped(|ui| {
            if ui.add(Button::new(i18n.tr("surprise-me")).icon(Icon::Dice).primary()).clicked() {
                act(ToolbarAction::Surprise);
            }
            ui.separator();
            for (icon, key, a) in [
                (Icon::New, "new-scene", ToolbarAction::New),
                (Icon::Open, "open-scene", ToolbarAction::Open),
                (Icon::Save, "save-scene", ToolbarAction::Save),
                (Icon::Picture, "export-image", ToolbarAction::Export),
            ] {
                if ui.add(IconButton::new(icon, i18n.tr(key))).clicked() {
                    act(a);
                }
            }
            ui.separator();
            if ui.add(IconButton::new(Icon::Undo, i18n.tr("undo")).enabled(self.can_undo)).clicked() {
                act(ToolbarAction::Undo);
            }
            if ui.add(IconButton::new(Icon::Redo, i18n.tr("redo")).enabled(self.can_redo)).clicked() {
                act(ToolbarAction::Redo);
            }
            ui.separator();
            for (f, key) in
                [(Filter::None, "filter-none"), (Filter::Dither, "filter-dither"), (Filter::Crt, "filter-crt")]
            {
                if ui.add(Button::new(i18n.tr(key)).selected(*self.filter == f)).clicked() {
                    *self.filter = f;
                }
            }
            ui.separator();
            let theme_name =
                self.themes.iter().find(|(id, _)| id == self.theme).map_or(self.theme, |(_, n)| n.as_str());
            egui::ComboBox::from_label(i18n.tr("theme")).selected_text(theme_name).show_ui(ui, |ui| {
                for (id, name) in self.themes {
                    if ui.selectable_label(id == self.theme, name).clicked() {
                        act(ToolbarAction::Theme(id.clone()));
                    }
                }
            });
            let lang_name = LANGUAGES.iter().find(|(c, _)| *c == i18n.lang()).map_or("", |(_, n)| *n);
            egui::ComboBox::from_label(i18n.tr("language")).selected_text(lang_name).show_ui(ui, |ui| {
                for (code, name) in LANGUAGES {
                    if ui.selectable_label(code == i18n.lang(), name).clicked() {
                        act(ToolbarAction::Language(code.to_owned()));
                    }
                }
            });
        });
        action
    }
}
