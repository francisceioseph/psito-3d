//! The UI kit of Psito 3D, in atomic design.
//!
//! - [`theme`]: design tokens and theme packs. Nothing below uses a literal
//!   colour or size; everything reads the theme in effect, so a theme pack
//!   (built in or from a plugin) restyles every widget.
//! - [`atoms`]: buttons, sliders, switches, icons.
//! - [`molecules`]: a few atoms working together (a labelled slider, the
//!   sun dial, the seed field).
//! - [`organisms`]: whole panels bound to the scene (sky, terrain, objects,
//!   toolbar, status bar, viewport).
//! - [`templates`]: where the panels go.
//!
//! The pages, which put real data into templates, live in the app.

pub mod atoms;
pub mod i18n;
pub mod molecules;
pub mod organisms;
pub mod templates;
pub mod theme;

pub use i18n::I18n;
