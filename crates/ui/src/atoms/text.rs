//! Text atoms.

use egui::{RichText, Ui};

use crate::theme;

/// A section heading.
pub fn heading(ui: &mut Ui, text: impl Into<String>) -> egui::Response {
    let t = theme::current(ui.ctx());
    ui.label(RichText::new(text.into()).size(t.typography.heading).strong().color(t.c(|p| p.text)))
}

/// A muted caption.
pub fn caption(ui: &mut Ui, text: impl Into<String>) -> egui::Response {
    let t = theme::current(ui.ctx());
    ui.label(RichText::new(text.into()).size(t.typography.small).color(t.c(|p| p.text_muted)))
}

/// A small value next to a control, in tabular figures.
pub fn value_text(ui: &mut Ui, text: impl Into<String>) -> egui::Response {
    let t = theme::current(ui.ctx());
    ui.label(RichText::new(text.into()).size(t.typography.small).monospace().color(t.c(|p| p.text)))
}
