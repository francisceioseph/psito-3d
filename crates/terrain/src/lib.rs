//! Heightfields and the terrain generators of Psito 3D.
//!
//! Every way of making a terrain is a [`TerrainGenerator`] (the Strategy
//! pattern): the three built in here, and the ones that plugins write in Lua
//! (in `psito3d-plugins`). The editor only talks to the trait, so a plugin
//! generator is exactly as good a citizen as a built-in one.
//!
//! The [`noise`] module is the single source of noise: the built-in
//! generators use it, and the Lua API hands the same functions to plugins.

mod builtin;
mod generator;
mod heightfield;
pub mod noise;

pub use builtin::builtin_generators;
pub use generator::{CancelToken, GenerateRequest, GeneratorInfo, TerrainError, TerrainGenerator, generate_terrain};
pub use heightfield::Heightfield;
