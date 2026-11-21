//! The scene code window: copy this world as a line of text, or paste
//! someone else's line to open theirs.

use egui::{Id, Modal, TextEdit};

use crate::I18n;
use crate::atoms::{Button, Icon, caption, heading};
use crate::theme;

/// The state of the window, kept by the page between frames.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SceneCodeDialog {
    /// Whether it is showing.
    pub open: bool,
    /// What the person pasted.
    pub input: String,
    /// Why the pasted code could not be opened, already translated.
    pub error: Option<String>,
    /// Whether the code was just copied, to say so.
    pub copied: bool,
}

impl SceneCodeDialog {
    /// Opens the window, clean.
    pub fn show_window(&mut self) {
        *self = Self { open: true, ..Self::default() };
    }

    /// Draws the window, if open. `code` is the current scene's code, and
    /// `uses_picture` says whether its terrain is a picture on this
    /// computer. Returns a pasted code to open.
    pub fn show(&mut self, ctx: &egui::Context, i18n: &I18n, code: &str, uses_picture: bool) -> Option<String> {
        if !self.open {
            return None;
        }
        let t = theme::current(ctx);
        let mut to_open = None;
        let response = Modal::new(Id::unique("psito3d.scene-code")).show(ctx, |ui| {
            ui.set_width(460.0f32.min(ctx.content_rect().width() - 48.0));
            heading(ui, i18n.tr("scene-code"));
            let share = caption(ui, i18n.tr("scene-code-share"));
            let mut shown = code.to_owned();
            // The caption above names the field for screen readers.
            ui.add(TextEdit::multiline(&mut shown).desired_rows(3).desired_width(f32::INFINITY).code_editor())
                .labelled_by(share.id);
            if uses_picture {
                ui.colored_label(t.c(|p| p.text_muted), i18n.tr("scene-code-picture"));
            }
            ui.horizontal(|ui| {
                if ui.add(Button::new(i18n.tr("copy-code")).icon(Icon::Share).primary()).clicked() {
                    ctx.copy_text(code.to_owned());
                    self.copied = true;
                }
                if self.copied {
                    caption(ui, i18n.tr("code-copied"));
                }
            });

            ui.separator();
            let paste = caption(ui, i18n.tr("scene-code-paste"));
            let field = ui
                .add(
                    TextEdit::multiline(&mut self.input)
                        .desired_rows(2)
                        .desired_width(f32::INFINITY)
                        .hint_text(i18n.tr("scene-code-hint")),
                )
                .labelled_by(paste.id);
            if field.changed() {
                self.error = None;
            }
            if let Some(e) = &self.error {
                ui.colored_label(t.c(|p| p.danger), e);
            }
            ui.horizontal(|ui| {
                let has_input = !self.input.trim().is_empty();
                if ui.add(Button::new(i18n.tr("open-code")).enabled(has_input)).clicked() {
                    to_open = Some(self.input.clone());
                }
                if ui.add(Button::new(i18n.tr("close"))).clicked() {
                    self.open = false;
                }
            });
        });
        if response.should_close() {
            self.open = false;
        }
        to_open
    }
}
