//! Themes: tokens, theme packs, and putting a theme into effect.
//!
//! A theme pack is a TOML file (see `docs/THEMES.md`). Two ship with the
//! program, and plugins can bring more. [`install`] stores the theme in the
//! egui context, where every widget of this crate reads it with [`current`],
//! and restyles egui's own widgets to match.

mod color;
mod tokens;

use std::path::Path;
use std::sync::Arc;

use egui::{Color32, CornerRadius, FontId, Stroke, TextStyle};

pub use color::Rgba;
pub use tokens::{ButtonStyle, Components, Metrics, Palette, SliderStyle, Theme, Typography};

/// What can go wrong reading a theme pack.
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    /// The file could not be read.
    #[error("could not read the theme: {0}")]
    Io(#[from] std::io::Error),
    /// The file is not a valid theme.
    #[error("invalid theme: {0}")]
    Parse(String),
}

impl Theme {
    /// Reads a theme pack and checks the sizes are sensible.
    pub fn from_toml(text: &str) -> Result<Self, ThemeError> {
        let theme: Self = toml::from_str(text).map_err(|e| ThemeError::Parse(e.to_string()))?;
        let m = &theme.metrics;
        let t = &theme.typography;
        let ok = |v: f32, lo: f32, hi: f32| v.is_finite() && (lo..=hi).contains(&v);
        if !(ok(m.radius, 0.0, 30.0) && ok(m.bevel, 0.0, 6.0) && ok(m.stroke, 0.0, 6.0) && ok(m.spacing, 2.0, 24.0))
            || !ok(m.control_height, 16.0, 48.0)
            || !(ok(t.body, 8.0, 28.0) && ok(t.heading, 8.0, 36.0) && ok(t.small, 8.0, 24.0))
        {
            return Err(ThemeError::Parse(format!("{}: a size is out of range", theme.id)));
        }
        Ok(theme)
    }

    /// Reads a theme pack file.
    pub fn from_file(path: &Path) -> Result<Self, ThemeError> {
        Self::from_toml(&std::fs::read_to_string(path)?)
    }

    /// The theme the program starts with.
    pub fn default_theme() -> Self {
        builtin_themes().into_iter().next().expect("there are built-in themes")
    }
}

/// The themes that ship with the program, the default first.
pub fn builtin_themes() -> Vec<Theme> {
    [include_str!("../../themes/dusk.toml"), include_str!("../../themes/daylight.toml")]
        .into_iter()
        .map(|text| Theme::from_toml(text).expect("the built-in themes are valid"))
        .collect()
}

fn theme_id() -> egui::Id {
    egui::Id::unique("psito3d.theme")
}

/// The theme in effect, or the default if none was installed.
pub fn current(ctx: &egui::Context) -> Arc<Theme> {
    ctx.data(|d| d.get_temp::<Arc<Theme>>(theme_id())).unwrap_or_else(|| Arc::new(Theme::default_theme()))
}

/// Puts `theme` into effect, for this crate's widgets and egui's own.
pub fn install(ctx: &egui::Context, theme: Theme) {
    let dark = theme.dark;
    let theme = Arc::new(theme);
    ctx.data_mut(|d| d.insert_temp(theme_id(), Arc::clone(&theme)));
    ctx.set_theme(if dark { egui::Theme::Dark } else { egui::Theme::Light });
    ctx.all_styles_mut(|style| apply_to_style(&theme, style));
}

fn apply_to_style(theme: &Theme, style: &mut egui::Style) {
    let c = |pick: fn(&Palette) -> Rgba| -> Color32 { theme.c(pick) };
    let m = &theme.metrics;
    let radius = CornerRadius::same(m.radius.round() as u8);

    let mut visuals = if theme.dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    visuals.override_text_color = Some(c(|p| p.text));
    visuals.panel_fill = c(|p| p.panel);
    visuals.window_fill = c(|p| p.panel);
    visuals.extreme_bg_color = c(|p| p.sunken);
    visuals.faint_bg_color = c(|p| p.raised);
    visuals.window_stroke = Stroke::new(m.stroke, c(|p| p.line));
    visuals.window_corner_radius = radius;
    visuals.menu_corner_radius = radius;
    visuals.selection.bg_fill = c(|p| p.accent);
    visuals.selection.stroke = Stroke::new(m.stroke, c(|p| p.accent_text));
    visuals.hyperlink_color = c(|p| p.accent);
    visuals.error_fg_color = c(|p| p.danger);
    for (w, fill) in [
        (&mut visuals.widgets.noninteractive, c(|p| p.panel)),
        (&mut visuals.widgets.inactive, c(|p| p.raised)),
        (&mut visuals.widgets.hovered, c(|p| p.raised)),
        (&mut visuals.widgets.active, c(|p| p.accent)),
        (&mut visuals.widgets.open, c(|p| p.raised)),
    ] {
        w.bg_fill = fill;
        w.weak_bg_fill = fill;
        w.corner_radius = radius;
        w.bg_stroke = Stroke::new(m.stroke, c(|p| p.line));
        w.fg_stroke = Stroke::new(m.stroke, c(|p| p.text));
    }
    visuals.widgets.hovered.bg_stroke = Stroke::new(m.stroke, c(|p| p.accent));
    visuals.widgets.active.fg_stroke = Stroke::new(m.stroke, c(|p| p.accent_text));
    style.visuals = visuals;

    style.spacing.item_spacing = egui::vec2(m.spacing, m.spacing * 0.75);
    style.spacing.button_padding = egui::vec2(m.spacing * 1.25, m.spacing * 0.5);
    style.spacing.interact_size.y = m.control_height;

    let t = &theme.typography;
    for (text_style, size) in [
        (TextStyle::Body, t.body),
        (TextStyle::Button, t.body),
        (TextStyle::Heading, t.heading),
        (TextStyle::Small, t.small),
        (TextStyle::Monospace, t.small),
    ] {
        let family = if text_style == TextStyle::Monospace {
            egui::FontFamily::Monospace
        } else {
            egui::FontFamily::Proportional
        };
        style.text_styles.insert(text_style, FontId::new(size, family));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_themes_load() {
        let themes = builtin_themes();
        assert!(themes.len() >= 2);
        assert!(themes.iter().any(|t| t.dark) && themes.iter().any(|t| !t.dark));
    }

    #[test]
    fn unknown_keys_and_bad_sizes_are_refused() {
        let good = include_str!("../../themes/dusk.toml");
        assert!(Theme::from_toml(&good.replace("[metrics]", "[metrics]\nwobble = 1.0")).is_err());
        assert!(Theme::from_toml(&good.replace("radius = 6.0", "radius = 99.0")).is_err());
    }

    #[test]
    fn install_restyles_egui_and_is_readable() {
        let ctx = egui::Context::default();
        let theme = builtin_themes().remove(1);
        let panel: Color32 = theme.colors.panel.into();
        install(&ctx, theme.clone());
        assert_eq!(current(&ctx).id, theme.id);
        assert_eq!(ctx.global_style().visuals.panel_fill, panel);
    }
}
