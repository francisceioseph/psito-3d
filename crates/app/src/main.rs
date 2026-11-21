//! Starts Psito 3D.

use psito3d_app::Psito3dApp;

fn main() -> eframe::Result {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,wgpu_core=warn,wgpu_hal=warn,naga=warn"),
    )
    .init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Psito 3D")
            .with_app_id("org.psito.psito3d")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([760.0, 520.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native("Psito 3D", options, Box::new(|cc| Ok(Box::new(Psito3dApp::new(cc)))))
}
