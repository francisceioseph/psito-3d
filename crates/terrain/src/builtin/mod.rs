//! The generators that ship with the program.
//!
//! Their descriptions are TOML files in the same format plugins use, so
//! the names and labels are translated in data and not in code.

use std::sync::Arc;

use psito3d_scene::ParamValues;

use crate::noise::{Fractal, fbm, ridged, smoothstep};
use crate::{CancelToken, GenerateRequest, GeneratorInfo, Heightfield, TerrainError, TerrainGenerator};

#[derive(Debug, Clone, Copy)]
enum Kind {
    Hills,
    Mountains,
    Island,
}

struct Builtin {
    info: GeneratorInfo,
    kind: Kind,
}

/// Folds a 64-bit seed into the 32 bits the noise takes.
fn seed32(seed: u64) -> u32 {
    (seed ^ (seed >> 32)) as u32
}

fn param(params: &ParamValues, id: &str, fallback: f32) -> f32 {
    params.get(id).map_or(fallback, |v| v.as_f32(fallback))
}

impl TerrainGenerator for Builtin {
    fn info(&self) -> &GeneratorInfo {
        &self.info
    }

    fn generate(&self, req: &GenerateRequest, cancel: &CancelToken) -> Result<Heightfield, TerrainError> {
        let seed = seed32(req.seed);
        let p = &req.params;
        let height = param(p, "height", 0.8);
        let is_cancelled = || cancel.is_cancelled();
        match self.kind {
            Kind::Hills => {
                let scale = param(p, "scale", 3.0);
                let f = Fractal { gain: param(p, "roughness", 0.5), ..Default::default() };
                Heightfield::from_fn(req.resolution, is_cancelled, |u, v| {
                    let n = fbm(u * scale, v * scale, seed, f) * 0.5 + 0.5;
                    Ok(n.powf(1.3) * height)
                })
            }
            Kind::Mountains => {
                let scale = param(p, "scale", 2.5);
                let sharp = param(p, "sharpness", 1.6);
                let f = Fractal { octaves: 7, gain: 0.5, lacunarity: 2.1 };
                Heightfield::from_fn(req.resolution, is_cancelled, |u, v| {
                    Ok(ridged(u * scale, v * scale, seed, f).powf(sharp) * height)
                })
            }
            Kind::Island => {
                let size = param(p, "size", 0.7);
                let peaks = param(p, "peaks", 0.5);
                let f = Fractal { gain: param(p, "roughness", 0.5), ..Default::default() };
                let warp = Fractal { octaves: 3, ..Default::default() };
                Heightfield::from_fn(req.resolution, is_cancelled, |u, v| {
                    // The coast wobbles: the distance to the centre is bent by noise.
                    let r = (u * u + v * v).sqrt() / size + 0.3 * fbm(u * 2.0, v * 2.0, seed ^ 0x51, warp);
                    let mask = 1.0 - smoothstep(0.35, 1.0, r);
                    let hills = fbm(u * 3.0, v * 3.0, seed, f) * 0.5 + 0.5;
                    let ridges = ridged(u * 2.5, v * 2.5, seed ^ 0xA5, f);
                    let base = hills + (ridges - hills) * peaks;
                    Ok(mask * (0.3 + 0.7 * base) * height)
                })
            }
        }
    }
}

/// The generators that ship with the program, in the order of the menu.
pub fn builtin_generators() -> Vec<Arc<dyn TerrainGenerator>> {
    [
        (include_str!("hills.toml"), Kind::Hills),
        (include_str!("island.toml"), Kind::Island),
        (include_str!("mountains.toml"), Kind::Mountains),
    ]
    .into_iter()
    .map(|(text, kind)| {
        let info = GeneratorInfo::from_toml(text).expect("the built-in descriptions are valid");
        Arc::new(Builtin { info, kind }) as Arc<dyn TerrainGenerator>
    })
    .collect()
}
