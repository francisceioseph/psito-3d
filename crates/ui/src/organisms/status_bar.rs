//! The bar along the bottom.

use egui::Ui;

use crate::I18n;
use crate::atoms::caption;
use crate::theme;

/// The state of the terrain being built.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TerrainStatus {
    /// The terrain on screen is up to date.
    #[default]
    Ready,
    /// A new terrain is being built.
    Working,
    /// The last build failed; the message says why.
    Failed(String),
}

/// What the status bar shows.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Status {
    /// Samples averaged so far.
    pub samples: u32,
    /// Samples for a finished image.
    pub target: u32,
    /// The terrain.
    pub terrain: TerrainStatus,
    /// Plugins that did not load.
    pub plugin_errors: Vec<String>,
    /// A passing message, such as "Saved".
    pub message: Option<String>,
}

/// The status bar.
pub fn status_bar(ui: &mut Ui, i18n: &I18n, status: &Status) {
    let t = theme::current(ui.ctx());
    ui.horizontal(|ui| {
        let progress = status.samples as f32 / status.target.max(1) as f32;
        let text = if status.samples >= status.target {
            i18n.tr("status-done")
        } else {
            i18n.tr_args("status-samples", &[("done", status.samples.into()), ("total", status.target.into())])
        };
        ui.add(egui::ProgressBar::new(progress.min(1.0)).desired_width(230.0).fill(t.c(|p| p.accent)).text(text));
        match &status.terrain {
            TerrainStatus::Ready => {}
            TerrainStatus::Working => {
                ui.spinner();
                caption(ui, i18n.tr("terrain-working"));
            }
            TerrainStatus::Failed(why) => {
                ui.colored_label(t.c(|p| p.danger), i18n.tr_args("terrain-failed", &[("reason", why.as_str().into())]));
            }
        }
        if !status.plugin_errors.is_empty() {
            ui.colored_label(
                t.c(|p| p.danger),
                i18n.tr_args("plugin-errors", &[("count", status.plugin_errors.len().into())]),
            )
            .on_hover_text(status.plugin_errors.join("\n"));
        }
        if let Some(m) = &status.message {
            caption(ui, m);
        }
    });
}
