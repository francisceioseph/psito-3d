//! The Psito 3D editor: the pages of the UI kit filled with a real scene,
//! the terrain built in the background, the renderer and the files.
//!
//! The parts with no window or GPU ([`editor`], [`surprise`], [`catalog`],
//! [`terrain_worker`], [`prefs`], [`wallpaper`]) are plain Rust and tested
//! on their own;
//! [`app`] wires them to eframe and wgpu.

pub mod app;
pub mod catalog;
pub mod editor;
pub mod export;
mod gpu_viewport;
pub mod pages;
pub mod paths;
pub mod prefs;
pub mod surprise;
pub mod terrain_worker;
pub mod wallpaper;

pub use app::{AppOptions, Psito3dApp};
