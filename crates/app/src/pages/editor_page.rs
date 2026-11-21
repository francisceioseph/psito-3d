//! The editor page: the only page of the program, for now.

use egui::Ui;
use psito3d_ui::I18n;
use psito3d_ui::organisms::{
    GeneratorChoice, Status, TerrainAction, Toolbar, ToolbarAction, ViewportResponse, atmosphere_panel, objects_panel,
    sky_panel, status_bar, terrain_panel, viewport,
};
use psito3d_ui::templates::EditorLayout;

use crate::editor::EditorState;

/// What the page needs to draw.
pub struct EditorPage<'a> {
    /// The translations.
    pub i18n: &'a I18n,
    /// The generators on offer.
    pub generators: &'a [GeneratorChoice],
    /// The themes on offer: id and name.
    pub themes: &'a [(String, String)],
    /// The theme in effect.
    pub theme: &'a str,
    /// The status bar.
    pub status: &'a Status,
    /// The rendered image, once there is a GPU.
    pub texture: Option<egui::TextureId>,
}

/// What happened on the page.
#[derive(Debug, Default)]
pub struct PageOutput {
    /// A toolbar button.
    pub toolbar: Option<ToolbarAction>,
    /// A terrain panel button.
    pub terrain: Option<TerrainAction>,
    /// The viewport's size and whether the camera is moving.
    pub viewport: Option<ViewportResponse>,
}

impl EditorPage<'_> {
    /// Draws the page, editing `editor` in place.
    pub fn show(&self, ui: &mut Ui, editor: &mut EditorState) -> PageOutput {
        let mut out = PageOutput::default();
        let i18n = self.i18n;
        let (can_undo, can_redo) = (editor.can_undo(), editor.can_redo());
        let scene = &mut editor.scene;
        let selected = &mut editor.selected;
        let generators = self.generators;
        let layout = EditorLayout::default();

        // The template's parts are separate closures, but they all edit the
        // same scene; a RefCell lets each borrow it in turn.
        let state = std::cell::RefCell::new((scene, selected));
        layout.show(
            ui,
            |ui| {
                let mut s = state.borrow_mut();
                let toolbar =
                    Toolbar { can_undo, can_redo, themes: self.themes, theme: self.theme, filter: &mut s.0.filter };
                out.toolbar = toolbar.show(ui, i18n);
            },
            |ui| {
                let mut s = state.borrow_mut();
                let (scene, selected) = &mut *s;
                sky_panel(ui, i18n, &mut scene.sky);
                ui.add_space(6.0);
                out.terrain = terrain_panel(ui, i18n, &mut scene.terrain, generators);
                ui.add_space(6.0);
                atmosphere_panel(ui, i18n, &mut scene.sea, &mut scene.fog);
                ui.add_space(6.0);
                objects_panel(ui, i18n, scene, selected);
            },
            |ui| status_bar(ui, i18n, self.status),
            |ui| {
                let mut s = state.borrow_mut();
                out.viewport = Some(viewport(ui, self.texture, &mut s.0.camera, &i18n.tr("viewport")));
            },
        );
        out
    }
}
