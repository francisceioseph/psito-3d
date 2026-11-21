//! The renderer of Psito 3D.
//!
//! It ray marches the scene on the GPU with wgpu, one sample per pixel per
//! frame, and averages the samples, so the image starts rough and sharpens
//! while the person looks at it (progressive rendering, as in Bryce).
//!
//! - [`gpu`] turns a [`Scene`](psito3d_scene::Scene) into the plain structs
//!   the shaders read. It has no GPU code, so it is tested on its own.
//! - [`SceneRenderer`] owns the textures and pipelines and draws into an
//!   sRGB texture that the editor shows and the exporter reads back.
//! - [`headless`] opens a GPU without a window, for tests and for the
//!   command line.

pub mod gpu;
pub mod headless;
mod quality;
mod renderer;

pub use quality::Quality;
pub use renderer::{OUTPUT_FORMAT, RenderError, SceneRenderer};

/// The WGSL source of the trace pass.
pub const TRACE_WGSL: &str = include_str!("shaders/trace.wgsl");
/// The WGSL source of the resolve pass.
pub const RESOLVE_WGSL: &str = include_str!("shaders/resolve.wgsl");
