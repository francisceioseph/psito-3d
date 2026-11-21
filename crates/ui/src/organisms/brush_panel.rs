//! The terrain brushes: pick a tool, then paint on the image.

use egui::Ui;
use psito3d_scene::{BrushStroke, BrushTool, TerrainSettings};

use crate::I18n;
use crate::atoms::{Button, Icon, caption};
use crate::molecules::{ChoiceChips, LabeledSlider, section};

/// The brush the person holds. It is not part of the scene: only the
/// strokes it paints are.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrushSettings {
    /// The tool, or `None` when the mouse turns the camera.
    pub tool: Option<BrushTool>,
    /// The radius, in world units.
    pub radius: f32,
    /// How strong each touch is, from 0 to 1.
    pub strength: f32,
}

impl Default for BrushSettings {
    fn default() -> Self {
        Self { tool: None, radius: 8.0, strength: 0.5 }
    }
}

fn tool_key(tool: Option<BrushTool>) -> &'static str {
    match tool {
        None => "brush-off",
        Some(BrushTool::Raise) => "brush-raise",
        Some(BrushTool::Lower) => "brush-lower",
        Some(BrushTool::Smooth) => "brush-smooth",
        Some(BrushTool::Flatten) => "brush-flatten",
    }
}

/// The brushes section.
pub fn brush_panel(ui: &mut Ui, i18n: &I18n, brush: &mut BrushSettings, terrain: &mut TerrainSettings) {
    section(ui, &i18n.tr("section-brushes"), |ui| {
        let options = std::iter::once(None)
            .chain(BrushTool::ALL.into_iter().map(Some))
            .map(|t| (t, i18n.tr(tool_key(t))))
            .collect();
        ChoiceChips::new(&mut brush.tool, options).show(ui);
        if brush.tool.is_some() {
            caption(ui, i18n.tr("brush-hint"));
        }
        let (lo, hi) = BrushStroke::RADIUS_RANGE;
        ui.add(
            LabeledSlider::new(&mut brush.radius, lo.max(1.0)..=hi, i18n.tr("brush-size"))
                .format(|v| format!("{v:.0}")),
        );
        ui.add(LabeledSlider::new(&mut brush.strength, 0.05..=1.0, i18n.tr("brush-strength")));
        ui.horizontal(|ui| {
            caption(ui, i18n.tr_args("brush-strokes", &[("count", terrain.strokes.len().into())]));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let any = !terrain.strokes.is_empty();
                if ui.add(Button::new(i18n.tr("clear-strokes")).icon(Icon::Trash).enabled(any)).clicked() {
                    terrain.strokes.clear();
                }
            });
        });
    });
}
