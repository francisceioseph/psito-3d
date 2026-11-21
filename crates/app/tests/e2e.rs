//! The whole program, end to end: the real `Psito3dApp` with the real
//! renderer on a real (or software) GPU, driven through its accessible UI by
//! egui_kittest, without a window.
//!
//! These sit at the top of the test pyramid: few, slow, and checking that
//! the parts work together. The details are tested further down.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use psito3d_app::export::PictureSettings;
use psito3d_app::wallpaper::{WallpaperError, WallpaperSetter};
use psito3d_app::{AppOptions, Psito3dApp};
use psito3d_ui::organisms::TerrainStatus;

fn bundled_plugins() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../plugins")
}

/// Records the pictures it is asked to put on the desktop, and never
/// touches the real one.
#[derive(Debug, Default)]
struct FakeWallpaper {
    set: Mutex<Vec<PathBuf>>,
    fail: bool,
}

impl WallpaperSetter for FakeWallpaper {
    fn set(&self, path: &Path) -> Result<(), WallpaperError> {
        self.set.lock().unwrap().push(path.to_owned());
        if self.fail { Err(WallpaperError::Unsupported) } else { Ok(()) }
    }
}

struct Run {
    harness: Harness<'static, Psito3dApp>,
    wallpaper: Arc<FakeWallpaper>,
    config: tempfile::TempDir,
}

fn start(lang: &str) -> Run {
    start_sized(lang, egui::vec2(1280.0, 800.0))
}

fn start_sized(lang: &str, size: egui::Vec2) -> Run {
    start_with(lang, size, FakeWallpaper::default())
}

fn start_with(lang: &str, size: egui::Vec2, wallpaper: FakeWallpaper) -> Run {
    let config = tempfile::tempdir().unwrap();
    let prefs = config.path().join("prefs.json");
    std::fs::write(&prefs, format!("{{\"language\":\"{lang}\"}}")).unwrap();
    let wallpaper = Arc::new(wallpaper);
    let options = AppOptions {
        prefs_path: Some(prefs),
        plugin_dirs: vec![bundled_plugins()],
        data_dir: Some(config.path().join("data")),
        seed: Some(42),
        wallpaper: Arc::clone(&wallpaper) as Arc<dyn WallpaperSetter>,
        // Small and quick: the CI renders on the CPU.
        wallpaper_picture: PictureSettings { size: (160, 90), samples: 4 },
    };
    let harness =
        Harness::builder().with_size(size).wgpu().build_eframe(move |cc| Psito3dApp::with_options(cc, options));
    Run { harness, wallpaper, config }
}

/// Runs frames until the terrain is built and the image has some samples.
fn settle(h: &mut Harness<'static, Psito3dApp>) {
    let start = Instant::now();
    loop {
        h.run_steps(4);
        let s = h.state().status_shown();
        if s.terrain == TerrainStatus::Ready && s.samples >= 4 {
            return;
        }
        assert!(start.elapsed() < Duration::from_secs(60), "never settled: {s:?}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn snapshot(h: &mut Harness<'static, Psito3dApp>, name: &str) {
    // Kept for people to look at, not compared: the image differs per GPU.
    if let Ok(dir) = std::env::var("PSITO3D_SNAPSHOT_DIR") {
        let img = h.render().expect("renders");
        img.save(PathBuf::from(dir).join(format!("{name}.png"))).unwrap();
    }
}

#[test]
fn starts_renders_and_surprises() {
    let mut run = start("en");
    let h = &mut run.harness;
    settle(h);
    snapshot(h, "start");
    assert!(h.state().status_shown().plugin_errors.is_empty());

    let before = h.state().scene().clone();
    h.get_by_label("Surprise me").click();
    settle(h);
    assert_ne!(h.state().scene(), &before, "a new world");
    snapshot(h, "surprise");

    h.get_by_label("Undo").click();
    h.run_steps(2);
    assert_eq!(h.state().scene(), &before, "and undo brings the old one back");
}

#[test]
fn speaks_portuguese_and_switches_theme() {
    // Tall enough that the whole side panel fits, objects included.
    let mut run = start_sized("pt", egui::vec2(1280.0, 2600.0));
    let h = &mut run.harness;
    settle(h);
    assert_eq!(h.state().language(), "pt");
    h.get_by_label("Adicionar uma esfera").click();
    h.run_steps(2);
    assert_eq!(h.state().scene().objects.len(), 2, "the starter ball and the new one");

    h.get_by_label("Tema").click();
    h.run_steps(2);
    h.get_by_label("Dia claro").click();
    h.run_steps(2);
    assert_eq!(h.state().theme_id(), "psito.daylight");
    settle(h);
    snapshot(h, "daylight-pt");
}

#[test]
fn a_plugin_terrain_can_be_chosen() {
    let mut run = start("en");
    let h = &mut run.harness;
    settle(h);
    h.get_by_label("Shape").click();
    h.run_steps(2);
    h.get_by_label("Volcanic island").click();
    settle(h);
    let psito3d_scene::TerrainSource::Generator { id, params } = &h.state().scene().terrain.source else { panic!() };
    assert_eq!(id, "org.psito.volcanic-island");
    assert!(params.contains_key("crater"), "the plugin's parameters are shown and filled in");
    snapshot(h, "volcano");
}

/// Runs frames until the status bar shows `text`.
fn wait_for_message(h: &mut Harness<'static, Psito3dApp>, text: &str) {
    let start = Instant::now();
    loop {
        h.run_steps(4);
        if h.state().status_shown().message.as_deref().is_some_and(|m| m.contains(text)) {
            return;
        }
        assert!(start.elapsed() < Duration::from_secs(60), "never said {text:?}: {:?}", h.state().status_shown());
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn the_picture_becomes_the_wallpaper() {
    let mut run = start("en");
    let h = &mut run.harness;
    settle(h);
    h.get_by_label("Use as wallpaper").click();
    wait_for_message(h, "Wallpaper changed");
    let set = run.wallpaper.set.lock().unwrap().clone();
    assert_eq!(set.len(), 1);
    assert!(set[0].starts_with(run.config.path().join("data/wallpapers")));
    let img = image::open(&set[0]).expect("a real PNG was written");
    assert_eq!((img.width(), img.height()), (160, 90));
}

#[test]
fn a_desktop_that_cannot_be_changed_gets_the_picture_path() {
    let mut run = start_with("pt", egui::vec2(1280.0, 800.0), FakeWallpaper { fail: true, ..Default::default() });
    let h = &mut run.harness;
    settle(h);
    h.get_by_label("Usar como papel de parede").click();
    wait_for_message(h, "A imagem está em");
    let path = run.wallpaper.set.lock().unwrap()[0].clone();
    assert!(h.state().status_shown().message.as_deref().unwrap().contains(&path.display().to_string()));
    assert!(path.exists(), "the picture stays, so the person can set it by hand");
}
