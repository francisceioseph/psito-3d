//! The eframe application: wires the editor state, the page, the terrain
//! worker, the renderer and the files together.

use std::time::{Duration, Instant};

use eframe::egui_wgpu::RenderState;
use egui::{Key, KeyboardShortcut, Modifiers};
use psito3d_render::Quality;
use psito3d_scene::{Scene, SplitMix64, TerrainSettings, TerrainSource};
use psito3d_terrain::{Heightfield, TerrainError};
use psito3d_ui::I18n;
use psito3d_ui::organisms::{GeneratorChoice, Status, TerrainAction, TerrainStatus, ToolbarAction};
use psito3d_ui::theme;

use crate::catalog::Catalog;
use crate::editor::EditorState;
use crate::export::ExportJob;
use crate::gpu_viewport::GpuViewport;
use crate::pages::EditorPage;
use crate::paths;
use crate::prefs::Prefs;
use crate::surprise::surprise;
use crate::terrain_worker::TerrainWorker;

/// How long after the last change the renderer switches from the quick
/// preview to the clean image.
const SETTLE_TIME: Duration = Duration::from_millis(220);
/// The terrain resolution while a terrain slider is being dragged.
const DRAG_RESOLUTION: u32 = 192;
/// How long a passing message stays in the status bar.
const MESSAGE_TIME: Duration = Duration::from_secs(4);

/// Where the app looks for things; the defaults are the real places, and
/// tests point them at temporary folders.
#[derive(Debug, Clone)]
pub struct AppOptions {
    /// The preferences file; `None` keeps nothing between runs.
    pub prefs_path: Option<std::path::PathBuf>,
    /// The folders plugins are looked for in.
    pub plugin_dirs: Vec<std::path::PathBuf>,
    /// The seed of "Surprise me" and of rolled seeds; `None` uses the clock.
    pub seed: Option<u64>,
}

impl Default for AppOptions {
    fn default() -> Self {
        Self { prefs_path: Prefs::path(), plugin_dirs: paths::plugin_dirs(), seed: None }
    }
}

/// Psito 3D.
pub struct Psito3dApp {
    editor: EditorState,
    i18n: I18n,
    catalog: Catalog,
    choices: Vec<GeneratorChoice>,
    theme_names: Vec<(String, String)>,
    theme_id: String,
    prefs: Prefs,
    prefs_path: Option<std::path::PathBuf>,
    worker: TerrainWorker,
    terrain_status: TerrainStatus,
    heightfield: Option<Heightfield>,
    gpu: Option<GpuViewport>,
    export: Option<ExportJob>,
    previous_scene: Scene,
    last_change: Instant,
    message: Option<(String, Instant)>,
    rng: SplitMix64,
    last_status: Status,
}

fn system_language() -> String {
    sys_locale::get_locale().unwrap_or_else(|| "en".to_owned())
}

impl Psito3dApp {
    /// Starts the editor with the starter scene.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::with_options(cc, AppOptions::default())
    }

    /// Starts the editor with the starter scene, looking for things where
    /// `options` says.
    pub fn with_options(cc: &eframe::CreationContext<'_>, options: AppOptions) -> Self {
        let prefs = options.prefs_path.as_deref().map(Prefs::load).unwrap_or_default();
        let catalog = Catalog::load(&options.plugin_dirs);
        let i18n = I18n::new(prefs.language.as_deref().unwrap_or(&system_language()));
        let theme = prefs
            .theme
            .as_ref()
            .and_then(|id| catalog.themes.iter().find(|t| &t.id == id))
            .cloned()
            .unwrap_or_else(theme::Theme::default_theme);
        let theme_id = theme.id.clone();
        theme::install(&cc.egui_ctx, theme);

        let seed = options.seed.unwrap_or_else(|| {
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64)
        });
        let scene = Scene::starter();
        let mut app = Self {
            choices: catalog.choices(i18n.lang()),
            theme_names: catalog.theme_names(i18n.lang()),
            worker: TerrainWorker::new(catalog.generators.clone()),
            gpu: cc.wgpu_render_state.as_ref().map(GpuViewport::new),
            editor: EditorState::new(scene.clone()),
            previous_scene: scene,
            catalog,
            i18n,
            theme_id,
            prefs,
            prefs_path: options.prefs_path,
            terrain_status: TerrainStatus::Ready,
            heightfield: None,
            export: None,
            last_change: Instant::now(),
            message: None,
            rng: SplitMix64::new(seed),
            last_status: Status::default(),
        };
        if app.gpu.is_none() {
            log::error!("eframe gave no wgpu device: the scene cannot be drawn");
        }
        app.request_terrain(&cc.egui_ctx, false);
        app
    }

    fn say(&mut self, text: String) {
        self.message = Some((text, Instant::now()));
    }

    fn save_prefs(&self) {
        if let Some(path) = &self.prefs_path
            && let Err(e) = self.prefs.save(path)
        {
            log::warn!("could not save the preferences: {e}");
        }
    }

    /// Asks the worker for the scene's terrain; at a low resolution while
    /// the person is dragging, so the preview keeps up.
    fn request_terrain(&mut self, ctx: &egui::Context, dragging: bool) {
        let mut settings: TerrainSettings = self.editor.scene.terrain.clone();
        if dragging {
            settings.resolution = settings.clamped_resolution().min(DRAG_RESOLUTION);
        }
        let ctx = ctx.clone();
        self.worker.request(&settings, self.editor.base_dir(), move || ctx.request_repaint());
    }

    fn poll_terrain(&mut self, rs: Option<&RenderState>) {
        match self.worker.poll() {
            Some(Ok(field)) => {
                if let (Some(gpu), Some(rs)) = (&mut self.gpu, rs) {
                    gpu.set_heightfield(rs, &field);
                }
                self.heightfield = Some(field);
                self.terrain_status = TerrainStatus::Ready;
            }
            Some(Err(TerrainError::Cancelled)) | None => {}
            Some(Err(e)) => self.terrain_status = TerrainStatus::Failed(e.to_string()),
        }
        if self.worker.is_busy() {
            self.terrain_status = TerrainStatus::Working;
        }
    }

    fn set_language(&mut self, code: &str) {
        self.i18n = I18n::new(code);
        self.choices = self.catalog.choices(self.i18n.lang());
        self.theme_names = self.catalog.theme_names(self.i18n.lang());
        self.prefs.language = Some(code.to_owned());
        self.save_prefs();
    }

    fn set_theme(&mut self, ctx: &egui::Context, id: &str) {
        if let Some(t) = self.catalog.themes.iter().find(|t| t.id == id) {
            self.theme_id = t.id.clone();
            theme::install(ctx, t.clone());
            self.prefs.theme = Some(t.id.clone());
            self.save_prefs();
        }
    }

    fn handle_toolbar(&mut self, ctx: &egui::Context, rs: Option<&RenderState>, action: ToolbarAction) {
        match action {
            ToolbarAction::Surprise => {
                let seed = self.rng.next_u64();
                self.editor.scene = surprise(seed, &self.choices, &self.editor.scene);
                self.editor.selected = None;
            }
            ToolbarAction::New => {
                let mut scene = Scene::starter();
                scene.filter = self.editor.scene.filter;
                self.editor.scene = scene;
                self.editor.file = None;
                self.editor.selected = None;
            }
            ToolbarAction::Undo => {
                self.editor.undo();
            }
            ToolbarAction::Redo => {
                self.editor.redo();
            }
            ToolbarAction::Open => self.open(),
            ToolbarAction::Save => self.save(),
            ToolbarAction::Export => self.export(rs),
            ToolbarAction::Theme(id) => self.set_theme(ctx, &id),
            ToolbarAction::Language(code) => self.set_language(&code),
        }
    }

    fn scene_dialog(&self) -> rfd::FileDialog {
        let mut dialog = rfd::FileDialog::new().add_filter("Psito 3D", &["psito3d"]);
        if let Some(dir) = self.editor.base_dir() {
            dialog = dialog.set_directory(dir);
        }
        dialog
    }

    fn open(&mut self) {
        let Some(path) = self.scene_dialog().pick_file() else { return };
        match self.editor.open(&path) {
            Ok(()) => {}
            Err(e) => {
                let text = self.i18n.tr_args("open-failed", &[("reason", e.to_string().into())]);
                self.say(text);
            }
        }
    }

    fn save(&mut self) {
        let path = match &self.editor.file {
            Some(p) => p.clone(),
            None => match self.scene_dialog().set_file_name("scene.psito3d").save_file() {
                Some(p) => p,
                None => return,
            },
        };
        let text = match self.editor.save(&path) {
            Ok(()) => self.i18n.tr("saved"),
            Err(e) => self.i18n.tr_args("save-failed", &[("reason", e.to_string().into())]),
        };
        self.say(text);
    }

    fn export(&mut self, rs: Option<&RenderState>) {
        let (Some(rs), Some(field)) = (rs, &self.heightfield) else { return };
        let Some(path) = rfd::FileDialog::new().add_filter("PNG", &["png"]).set_file_name("psito-3d.png").save_file()
        else {
            return;
        };
        self.export = Some(ExportJob::start(rs, &self.editor.scene, field, path));
    }

    fn pick_picture(&mut self) {
        let Some(path) =
            rfd::FileDialog::new().add_filter("Image", &["png", "jpg", "jpeg", "bmp", "webp", "gif"]).pick_file()
        else {
            return;
        };
        self.editor.scene.terrain.source = TerrainSource::Image { path };
    }

    fn shortcuts(&mut self, ctx: &egui::Context, rs: Option<&RenderState>) {
        let cmd = Modifiers::COMMAND;
        let pressed = |m: Modifiers, k: Key| ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(m, k)));
        // Shift+Ctrl+Z before Ctrl+Z, which would also match it.
        if pressed(cmd | Modifiers::SHIFT, Key::Z) || pressed(cmd, Key::Y) {
            self.handle_toolbar(ctx, rs, ToolbarAction::Redo);
        } else if pressed(cmd, Key::Z) {
            self.handle_toolbar(ctx, rs, ToolbarAction::Undo);
        }
        if pressed(cmd, Key::S) {
            self.save();
        }
        if pressed(cmd, Key::O) {
            self.open();
        }
        if pressed(cmd, Key::E) {
            self.export(rs);
        }
    }

    fn status(&mut self) -> Status {
        if self.message.as_ref().is_some_and(|(_, at)| at.elapsed() > MESSAGE_TIME) {
            self.message = None;
        }
        let (samples, target) = self.gpu.as_ref().map_or((0, 1), GpuViewport::progress);
        let mut message = self.message.as_ref().map(|(m, _)| m.clone());
        if let Some(job) = &self.export {
            message = Some(format!("{} {:.0}%", self.i18n.tr("export-image"), job.progress() * 100.0));
        }
        Status {
            samples,
            target,
            terrain: self.terrain_status.clone(),
            plugin_errors: self.catalog.errors.clone(),
            message,
        }
    }
}

impl Psito3dApp {
    /// The scene being edited.
    pub fn scene(&self) -> &Scene {
        &self.editor.scene
    }

    /// What the status bar showed last frame.
    pub fn status_shown(&self) -> &Status {
        &self.last_status
    }

    /// The id of the theme in effect.
    pub fn theme_id(&self) -> &str {
        &self.theme_id
    }

    /// The language in use.
    pub fn language(&self) -> &str {
        self.i18n.lang()
    }
}

impl eframe::App for Psito3dApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let rs = frame.wgpu_render_state().cloned();
        let rs = rs.as_ref();

        self.poll_terrain(rs);
        self.shortcuts(&ctx, rs);

        let status = self.status();
        self.last_status = status.clone();
        let page = EditorPage {
            i18n: &self.i18n,
            generators: &self.choices,
            themes: &self.theme_names,
            theme: &self.theme_id,
            status: &status,
            texture: self.gpu.as_ref().map(GpuViewport::texture),
        };
        let out = page.show(ui, &mut self.editor);
        if let Some(action) = out.toolbar {
            self.handle_toolbar(&ctx, rs, action);
        }
        match out.terrain {
            Some(TerrainAction::PickPicture) => self.pick_picture(),
            Some(TerrainAction::RollSeed) => self.editor.scene.terrain.seed = self.rng.next_u64() & 0xFFFF_FFFF,
            None => {}
        }

        let pointer_down = ctx.input(|i| i.pointer.any_down());
        let camera_moving = out.viewport.is_some_and(|v| v.interacting);
        let changed = self.editor.scene != self.previous_scene;
        if changed || camera_moving {
            self.last_change = Instant::now();
            self.previous_scene = self.editor.scene.clone();
        }
        // Moving the camera is not an edit: it is not worth an undo step.
        self.editor.settle(pointer_down);
        self.request_terrain(&ctx, pointer_down && changed);

        let mut busy = self.worker.is_busy();
        if let (Some(gpu), Some(rs), Some(v)) = (&mut self.gpu, rs, out.viewport) {
            let quality = if self.last_change.elapsed() < SETTLE_TIME { Quality::Preview } else { Quality::Final };
            busy |= gpu.frame(rs, &self.editor.scene, v.pixels, quality);
            busy |= quality == Quality::Preview;
        }
        if let (Some(job), Some(rs)) = (&mut self.export, rs) {
            busy = true;
            if let Some(result) = job.step(rs) {
                self.export = None;
                let text = match result {
                    Ok(path) => format!("{}: {}", self.i18n.tr("exported"), path.display()),
                    Err(e) => self.i18n.tr_args("save-failed", &[("reason", e.into())]),
                };
                self.say(text);
            }
        }
        if busy || self.message.is_some() {
            ctx.request_repaint();
        }
    }
}
