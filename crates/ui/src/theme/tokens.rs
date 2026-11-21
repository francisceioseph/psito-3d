//! Design tokens: the values every widget draws with.
//!
//! No widget in this crate uses a literal colour or size. They read a
//! [`Theme`], which comes from a theme pack (TOML), so a pack, built in or
//! from a plugin, restyles the whole program.

use egui::Color32;
use psito3d_scene::LocalizedText;
use serde::Deserialize;

use super::color::Rgba;

/// The colours, by what each one is for.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Palette {
    /// Behind everything.
    pub background: Rgba,
    /// Panels and cards.
    pub panel: Rgba,
    /// Raised controls: buttons, the slider thumb.
    pub raised: Rgba,
    /// Sunken areas: slider grooves, text fields.
    pub sunken: Rgba,
    /// Body text.
    pub text: Rgba,
    /// Secondary text and captions.
    pub text_muted: Rgba,
    /// The accent: active controls, the selection, the sun.
    pub accent: Rgba,
    /// Text drawn on the accent.
    pub accent_text: Rgba,
    /// Thin lines and borders.
    pub line: Rgba,
    /// The light edge of a bevel.
    pub bevel_light: Rgba,
    /// The dark edge of a bevel.
    pub bevel_dark: Rgba,
    /// Errors.
    pub danger: Rgba,
}

/// How a button is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ButtonStyle {
    /// Raised, with a light top edge and a dark bottom edge.
    #[default]
    Bevel,
    /// A flat fill.
    Flat,
    /// A flat fill with fully round ends.
    Pill,
}

/// How a slider is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SliderStyle {
    /// A sunken groove with a raised thumb.
    #[default]
    Groove,
    /// A thin line with a round knob.
    Line,
}

/// Sizes and shapes.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Metrics {
    /// The corner radius of controls, in points.
    pub radius: f32,
    /// The width of a bevel edge; 0 draws everything flat.
    pub bevel: f32,
    /// The width of lines and borders.
    pub stroke: f32,
    /// The base spacing unit.
    pub spacing: f32,
    /// The height of a control (button, slider row).
    pub control_height: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Self { radius: 5.0, bevel: 1.5, stroke: 1.0, spacing: 8.0, control_height: 26.0 }
    }
}

/// Font sizes.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Typography {
    /// Body text.
    pub body: f32,
    /// Section headings.
    pub heading: f32,
    /// Captions and values.
    pub small: f32,
}

impl Default for Typography {
    fn default() -> Self {
        Self { body: 14.0, heading: 15.0, small: 12.0 }
    }
}

/// Per-component choices a pack may override.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Components {
    /// Buttons.
    pub button: ButtonStyle,
    /// Sliders.
    pub slider: SliderStyle,
}

/// A complete theme.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    /// The id, such as `psito.dusk`.
    pub id: String,
    /// The name in the theme menu.
    pub name: LocalizedText,
    /// Whether the background is dark, for egui's own widgets.
    #[serde(default)]
    pub dark: bool,
    /// The colours.
    pub colors: Palette,
    /// Sizes and shapes.
    #[serde(default)]
    pub metrics: Metrics,
    /// Font sizes.
    #[serde(default, rename = "type")]
    pub typography: Typography,
    /// Component styles.
    #[serde(default)]
    pub components: Components,
}

impl Theme {
    /// A colour of the palette, ready for egui.
    pub fn c(&self, pick: impl Fn(&Palette) -> Rgba) -> Color32 {
        pick(&self.colors).into()
    }
}
