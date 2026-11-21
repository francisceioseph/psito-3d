//! The `psito` table: everything a plugin can call.
//!
//! The noise functions are the same Rust code the built-in generators use,
//! so they are fast and give the same results everywhere. See
//! `docs/PLUGINS.md` for the reference.

use mlua::{Lua, Table};
use psito3d_terrain::noise::{self, Fractal};

use crate::API_VERSION;

fn fold_seed(seed: u64) -> u32 {
    (seed ^ (seed >> 32)) as u32
}

/// Reads the optional settings table of `fbm` and `ridged`.
fn fractal(opts: Option<&Table>, default_seed: u32) -> mlua::Result<(Fractal, u32)> {
    let d = Fractal::default();
    let Some(t) = opts else { return Ok((d, default_seed)) };
    let octaves: Option<u32> = t.get("octaves")?;
    let f = Fractal {
        octaves: octaves.unwrap_or(d.octaves).clamp(1, Fractal::MAX_OCTAVES),
        lacunarity: t.get::<Option<f32>>("lacunarity")?.unwrap_or(d.lacunarity),
        gain: t.get::<Option<f32>>("gain")?.unwrap_or(d.gain),
    };
    let seed = t.get::<Option<i64>>("seed")?.map_or(default_seed, |s| default_seed ^ fold_seed(s as u64));
    Ok((f, seed))
}

/// Puts the `psito` table in the globals, for a job with `seed`.
pub fn install(lua: &Lua, seed: u64) -> mlua::Result<()> {
    let base = fold_seed(seed);
    let psito = lua.create_table()?;
    psito.set("api", API_VERSION)?;
    psito.set("seed", seed as i64)?;

    let n = lua.create_table()?;
    n.set(
        "perlin",
        lua.create_function(move |_, (x, y, s): (f32, f32, Option<i64>)| {
            Ok(noise::perlin(x, y, s.map_or(base, |s| base ^ fold_seed(s as u64))))
        })?,
    )?;
    n.set(
        "fbm",
        lua.create_function(move |_, (x, y, opts): (f32, f32, Option<Table>)| {
            let (f, seed) = fractal(opts.as_ref(), base)?;
            Ok(noise::fbm(x, y, seed, f))
        })?,
    )?;
    n.set(
        "ridged",
        lua.create_function(move |_, (x, y, opts): (f32, f32, Option<Table>)| {
            let (f, seed) = fractal(opts.as_ref(), base)?;
            Ok(noise::ridged(x, y, seed, f))
        })?,
    )?;
    psito.set("noise", n)?;

    let m = lua.create_table()?;
    m.set("smoothstep", lua.create_function(|_, (a, b, x): (f32, f32, f32)| Ok(noise::smoothstep(a, b, x)))?)?;
    m.set("clamp", lua.create_function(|_, (x, lo, hi): (f64, f64, f64)| Ok(x.max(lo).min(hi)))?)?;
    m.set("mix", lua.create_function(|_, (a, b, t): (f64, f64, f64)| Ok(a + (b - a) * t))?)?;
    psito.set("math", m)?;

    lua.globals().set("psito", psito)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::{Limits, new_sandbox, run_chunk};
    use psito3d_terrain::CancelToken;

    fn eval(src: &str, seed: u64) -> f64 {
        let lua = new_sandbox(Limits::default(), CancelToken::new(), "t", seed).unwrap();
        install(&lua, seed).unwrap();
        lua.unpack::<f64>(run_chunk(&lua, src, "t").unwrap()).unwrap()
    }

    #[test]
    fn noise_matches_the_rust_functions() {
        let v = eval("return psito.noise.fbm(0.3, 0.7)", 5);
        assert!((v - noise::fbm(0.3, 0.7, 5, Fractal::default()) as f64).abs() < 1e-6);
    }

    #[test]
    fn options_and_seeds_change_the_noise() {
        let a = eval("return psito.noise.ridged(0.3, 0.7, { octaves = 2 })", 5);
        let b = eval("return psito.noise.ridged(0.3, 0.7, { octaves = 2, seed = 9 })", 5);
        assert_ne!(a, b);
        assert_ne!(eval("return psito.noise.perlin(0.3, 0.7)", 1), eval("return psito.noise.perlin(0.3, 0.7)", 2));
    }

    #[test]
    fn helpers() {
        assert_eq!(eval("return psito.math.clamp(5, 0, 1)", 1), 1.0);
        assert_eq!(eval("return psito.math.mix(0, 10, 0.25)", 1), 2.5);
        assert_eq!(eval("return psito.math.smoothstep(0, 1, 2)", 1), 1.0);
        assert_eq!(eval("return psito.api", 1), API_VERSION as f64);
    }
}
