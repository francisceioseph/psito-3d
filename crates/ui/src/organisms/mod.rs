//! Organisms: whole panels, bound to the scene model.
//!
//! Each organism edits a part of the [`Scene`](psito3d_scene::Scene) in
//! place and returns the actions it cannot do itself (opening a file
//! dialog, rolling a seed). It never touches the undo history: the editor
//! compares the scene before and after.

mod atmosphere_panel;
mod objects_panel;
mod param_form;
mod sky_panel;
mod status_bar;
mod terrain_panel;
mod toolbar;
mod viewport;

pub use atmosphere_panel::atmosphere_panel;
pub use objects_panel::objects_panel;
pub use param_form::param_form;
pub use sky_panel::sky_panel;
pub use status_bar::{Status, TerrainStatus, status_bar};
pub use terrain_panel::{GeneratorChoice, TerrainAction, terrain_panel};
pub use toolbar::{Toolbar, ToolbarAction};
pub use viewport::{ViewportResponse, viewport};
