//! Starts Psito 3D.

// No console window behind the program on Windows.
#![cfg_attr(windows, windows_subsystem = "windows")]

use psito3d_app::Psito3dApp;

/// The icon, also used by the packages (`assets/icon.png`).
const ICON_PNG: &[u8] = include_bytes!("../../../assets/icon.png");

fn icon() -> Option<egui::IconData> {
    let img = image::load_from_memory(ICON_PNG).ok()?.into_rgba8();
    Some(egui::IconData { width: img.width(), height: img.height(), rgba: img.into_raw() })
}

fn main() -> eframe::Result {
    // The few command line flags, for packagers and for people who ask.
    match std::env::args().nth(1).as_deref() {
        Some("--version" | "-V") => {
            println!("psito-3d {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some("--help" | "-h") => {
            println!(
                "psito-3d {}\nMake landscapes like the 90s.\n\nUsage: psito-3d [--version | --help]",
                env!("CARGO_PKG_VERSION")
            );
            return Ok(());
        }
        _ => {}
    }
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,wgpu_core=warn,wgpu_hal=warn,naga=warn"),
    )
    .init();
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Psito 3D")
        .with_app_id("org.psito.psito3d")
        .with_inner_size([1280.0, 800.0])
        .with_min_inner_size([760.0, 520.0]);
    if let Some(icon) = icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions { viewport, renderer: eframe::Renderer::Wgpu, ..Default::default() };
    eframe::run_native("Psito 3D", options, Box::new(|cc| Ok(Box::new(Psito3dApp::new(cc)))))
}
