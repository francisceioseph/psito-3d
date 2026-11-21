//! The panels, bound to a real scene, in both languages.

use egui::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use psito3d_scene::{ObjectId, Scene, Shape, TerrainSource, resolve_params};
use psito3d_ui::I18n;
use psito3d_ui::organisms::{
    GeneratorChoice, TerrainAction, Toolbar, ToolbarAction, objects_panel, sky_panel, terrain_panel,
};

struct State {
    scene: Scene,
    selected: Option<ObjectId>,
    i18n: I18n,
    terrain_action: Option<TerrainAction>,
}

fn state(lang: &str) -> State {
    State { scene: Scene::default(), selected: None, i18n: I18n::new(lang), terrain_action: None }
}

fn generators() -> Vec<GeneratorChoice> {
    psito3d_terrain_like_generators()
}

/// Two made-up generators with a slider and a toggle, so the form has
/// something to show without depending on the terrain crate.
fn psito3d_terrain_like_generators() -> Vec<GeneratorChoice> {
    use psito3d_scene::{LocalizedText, ParamSpec};
    vec![
        GeneratorChoice {
            id: "psito.hills".into(),
            name: "Hills".into(),
            description: String::new(),
            params: vec![ParamSpec::slider("height", LocalizedText::new("Height").with("pt", "Altura"), 0.0, 1.0, 0.5)],
        },
        GeneratorChoice {
            id: "test.toggle".into(),
            name: "Toggle".into(),
            description: String::new(),
            params: vec![ParamSpec::toggle("craters", LocalizedText::new("Craters"), true)],
        },
    ]
}

#[test]
fn adding_objects_selects_them() {
    let mut harness = Harness::builder().with_size(egui::vec2(360.0, 900.0)).build_ui_state(
        |ui, s: &mut State| {
            objects_panel(ui, &s.i18n, &mut s.scene, &mut s.selected);
        },
        state("en"),
    );
    harness.get_by_label("Add a ball").click();
    harness.run();
    harness.get_by_label("Add a ring").click();
    harness.run();
    let s = harness.state();
    assert_eq!(s.scene.objects.len(), 2);
    assert_eq!(s.scene.objects[1].shape, Shape::Torus);
    assert_eq!(s.selected, Some(s.scene.objects[1].id));
}

#[test]
fn material_presets_and_delete() {
    let mut harness = Harness::builder().with_size(egui::vec2(360.0, 1400.0)).build_ui_state(
        |ui, s: &mut State| {
            objects_panel(ui, &s.i18n, &mut s.scene, &mut s.selected);
        },
        state("pt"),
    );
    harness.get_by_label("Adicionar uma caixa").click();
    harness.run();
    harness.get_by_label("Vidro").click();
    harness.run();
    assert_eq!(harness.state().scene.objects[0].material.transmission, 1.0);
    harness.get_by_label("Apagar objeto").click();
    harness.run();
    assert!(harness.state().scene.objects.is_empty());
    assert_eq!(harness.state().selected, None);
}

#[test]
fn hiding_an_object() {
    let mut s = state("en");
    s.scene.add_object(Shape::Cone, Default::default());
    let mut harness = Harness::builder().with_size(egui::vec2(360.0, 600.0)).build_ui_state(
        |ui, s: &mut State| {
            objects_panel(ui, &s.i18n, &mut s.scene, &mut s.selected);
        },
        s,
    );
    harness.get_by_label("Hide").click();
    harness.run();
    assert!(!harness.state().scene.objects[0].visible);
}

#[test]
fn the_terrain_form_follows_the_generator() {
    let mut s = state("pt");
    let gens = generators();
    s.scene.terrain.source = TerrainSource::Generator { id: "psito.hills".into(), params: Default::default() };
    let mut harness = Harness::builder().with_size(egui::vec2(360.0, 900.0)).build_ui_state(
        move |ui, s: &mut State| {
            if let Some(a) = terrain_panel(ui, &s.i18n, &mut s.scene.terrain, &gens) {
                s.terrain_action = Some(a);
            }
        },
        s,
    );
    harness.run();
    // The parameter label comes from the description, in Portuguese.
    harness.get_by_role_and_label(Role::Slider, "Altura");
    // Showing the form does not touch the scene.
    let TerrainSource::Generator { params, .. } = &harness.state().scene.terrain.source else { panic!() };
    assert!(params.is_empty());
    let _ = resolve_params;
    harness.get_by_label("Sortear outra semente").click();
    harness.run();
    assert_eq!(harness.state().terrain_action, Some(TerrainAction::RollSeed));
}

#[test]
fn the_toolbar_reports_actions_and_disables_undo() {
    let themes = vec![("psito.dusk".to_owned(), "Dusk".to_owned())];
    let mut harness = Harness::builder().with_size(egui::vec2(1200.0, 80.0)).build_ui_state(
        move |ui, (s, actions): &mut (State, Vec<ToolbarAction>)| {
            let toolbar = Toolbar {
                can_undo: false,
                can_redo: true,
                themes: &themes,
                theme: "psito.dusk",
                filter: &mut s.scene.filter,
                orbiting: false,
            };
            if let Some(a) = toolbar.show(ui, &s.i18n) {
                actions.push(a);
            }
        },
        (state("en"), Vec::new()),
    );
    harness.get_by_label("Surprise me").click();
    harness.run();
    harness.get_by_label("Undo").click();
    harness.run();
    harness.get_by_label("Redo").click();
    harness.run();
    harness.get_by_label("256 colours").click();
    harness.run();
    let (s, actions) = harness.state();
    assert_eq!(actions, &[ToolbarAction::Surprise, ToolbarAction::Redo]);
    assert_eq!(s.scene.filter, psito3d_scene::Filter::Dither);
}

#[test]
fn every_control_has_a_name() {
    let mut s = state("en");
    let id = s.scene.add_object(Shape::Sphere, Default::default()).unwrap();
    s.selected = Some(id);
    let gens = generators();
    let harness = Harness::builder().with_size(egui::vec2(380.0, 3000.0)).build_ui_state(
        move |ui, s: &mut State| {
            sky_panel(ui, &s.i18n, &mut s.scene.sky);
            terrain_panel(ui, &s.i18n, &mut s.scene.terrain, &gens);
            objects_panel(ui, &s.i18n, &mut s.scene, &mut s.selected);
        },
        s,
    );
    for role in [Role::Button, Role::Slider, Role::Switch, Role::ColorWell] {
        for node in harness.query_all_by_role(role) {
            let label = node.accesskit_node().label().unwrap_or_default();
            assert!(!label.trim().is_empty(), "a {role:?} has no name");
        }
    }
}

#[test]
fn the_scene_code_window_opens_a_pasted_code() {
    use psito3d_ui::organisms::SceneCodeDialog;
    let mut dialog = SceneCodeDialog::default();
    dialog.show_window();
    let mut harness = Harness::builder().with_size(egui::vec2(800.0, 600.0)).build_ui_state(
        |ui, (d, opened, i18n): &mut (SceneCodeDialog, Option<String>, I18n)| {
            if let Some(code) = d.show(ui.ctx(), i18n, "p3d1.current", false) {
                *opened = Some(code);
            }
        },
        (dialog, None, I18n::new("en")),
    );
    harness.run();
    let field = harness.get_by_role_and_label(Role::MultilineTextInput, "Paste a code to open its world");
    field.focus();
    harness.run();
    harness.get_by_role_and_label(Role::MultilineTextInput, "Paste a code to open its world").type_text("p3d1.abc");
    harness.run();
    harness.get_by_label("Open").click();
    harness.run();
    assert_eq!(harness.state().1.as_deref(), Some("p3d1.abc"));
}
