//! A terrain generator written in Lua.

use std::sync::Arc;

use mlua::{Function, Value};
use psito3d_scene::ParamValue;
use psito3d_terrain::{CancelToken, GenerateRequest, GeneratorInfo, Heightfield, TerrainError, TerrainGenerator};

use crate::api;
use crate::sandbox::{Limits, new_sandbox, run_chunk};

/// A terrain generator whose heights come from a Lua function.
///
/// The script returns either a function or a table with a `height`
/// function. It is called once per sample as `height(x, y, params)`, with `x`
/// and `y` from -1 to 1, and returns the height from 0 to 1:
///
/// ```lua
/// return {
///   height = function(x, y, p)
///     return psito.noise.fbm(x * 3, y * 3) * 0.5 + 0.5
///   end,
/// }
/// ```
///
/// Each generation gets a fresh sandbox, so nothing leaks from one run to
/// the next and the result depends only on the request.
#[derive(Debug, Clone)]
pub struct LuaTerrainGenerator {
    info: GeneratorInfo,
    source: Arc<str>,
    chunk_name: String,
    limits: Limits,
}

impl LuaTerrainGenerator {
    /// A generator running `source`. `chunk_name` names it in error messages.
    pub fn new(
        info: GeneratorInfo,
        source: impl Into<Arc<str>>,
        chunk_name: impl Into<String>,
        limits: Limits,
    ) -> Self {
        Self { info, source: source.into(), chunk_name: chunk_name.into(), limits }
    }
}

/// The message the person reads: the innermost Lua error, without the
/// traceback of the callbacks around it.
fn lua_message(err: &mlua::Error) -> String {
    match err {
        mlua::Error::CallbackError { cause, .. } => lua_message(cause),
        mlua::Error::RuntimeError(m) | mlua::Error::SyntaxError { message: m, .. } => {
            m.lines().next().unwrap_or(m).to_owned()
        }
        other => other.to_string().lines().next().unwrap_or_default().to_owned(),
    }
}

impl TerrainGenerator for LuaTerrainGenerator {
    fn info(&self) -> &GeneratorInfo {
        &self.info
    }

    fn generate(&self, req: &GenerateRequest, cancel: &CancelToken) -> Result<Heightfield, TerrainError> {
        let fail = |e: mlua::Error| {
            if cancel.is_cancelled() { TerrainError::Cancelled } else { TerrainError::Plugin(lua_message(&e)) }
        };
        let lua = new_sandbox(self.limits, cancel.clone(), &self.info.id, req.seed).map_err(fail)?;
        api::install(&lua, req.seed).map_err(fail)?;
        let height: Function = match run_chunk(&lua, &self.source, &self.chunk_name).map_err(fail)? {
            Value::Function(f) => f,
            Value::Table(t) => t
                .get("height")
                .map_err(|_| TerrainError::Plugin(format!("{}: the table has no height function", self.chunk_name)))?,
            _ => {
                return Err(TerrainError::Plugin(format!(
                    "{}: the script must return a height function or a table with one",
                    self.chunk_name
                )));
            }
        };
        let params = lua.create_table().map_err(fail)?;
        for (key, value) in &req.params {
            let v = match value {
                ParamValue::Number(n) => Value::Number(*n as f64),
                ParamValue::Bool(b) => Value::Boolean(*b),
                ParamValue::Choice(s) => Value::String(lua.create_string(s).map_err(fail)?),
            };
            params.set(key.as_str(), v).map_err(fail)?;
        }
        Heightfield::from_fn(
            req.resolution,
            || cancel.is_cancelled(),
            |x, y| height.call::<f64>((x, y, &params)).map(|h| h as f32).map_err(fail),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use psito3d_scene::ParamValues;
    use std::time::Duration;

    fn generator(src: &str) -> LuaTerrainGenerator {
        let info = GeneratorInfo {
            id: "t.lua".into(),
            name: Default::default(),
            description: Default::default(),
            params: vec![],
        };
        LuaTerrainGenerator::new(info, src, "terrain.lua", Limits::default())
    }

    fn request(params: ParamValues) -> GenerateRequest {
        GenerateRequest { seed: 3, resolution: 16, params }
    }

    #[test]
    fn a_function_or_a_table() {
        let f = generator("return function(x, y) return (x + 1) / 2 end");
        let t = generator("return { height = function(x, y) return (x + 1) / 2 end }");
        let a = f.generate(&request(ParamValues::new()), &CancelToken::new()).unwrap();
        let b = t.generate(&request(ParamValues::new()), &CancelToken::new()).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.get(0, 5), 0.0);
        assert_eq!(a.get(15, 5), 1.0);
    }

    #[test]
    fn params_reach_the_script() {
        let g = generator("return function(x, y, p) if p.flat then return p.level end return 0 end");
        let mut params = ParamValues::new();
        params.insert("flat".into(), ParamValue::Bool(true));
        params.insert("level".into(), ParamValue::Number(0.25));
        let h = g.generate(&request(params), &CancelToken::new()).unwrap();
        assert!(h.samples().iter().all(|&v| v == 0.25));
    }

    #[test]
    fn errors_name_the_file_and_line() {
        let g = generator("return function(x, y)\n  error('boom')\nend");
        let err = g.generate(&request(ParamValues::new()), &CancelToken::new()).unwrap_err();
        let TerrainError::Plugin(msg) = err else { panic!("{err:?}") };
        assert!(msg.contains("terrain.lua:2") && msg.contains("boom"), "{msg}");
    }

    #[test]
    fn wrong_returns_are_explained() {
        let err = generator("return 42").generate(&request(ParamValues::new()), &CancelToken::new()).unwrap_err();
        assert!(matches!(err, TerrainError::Plugin(m) if m.contains("must return")));
        let err = generator("return function() return 'tall' end")
            .generate(&request(ParamValues::new()), &CancelToken::new())
            .unwrap_err();
        assert!(matches!(err, TerrainError::Plugin(_)));
    }

    #[test]
    fn slow_scripts_hit_the_time_limit() {
        let info = GeneratorInfo {
            id: "t.slow".into(),
            name: Default::default(),
            description: Default::default(),
            params: vec![],
        };
        let limits = Limits { time: Duration::from_millis(150), ..Default::default() };
        let g = LuaTerrainGenerator::new(info, "return function() while true do end end", "slow.lua", limits);
        let err = g.generate(&request(ParamValues::new()), &CancelToken::new()).unwrap_err();
        assert!(matches!(err, TerrainError::Plugin(m) if m.contains("out of time")));
    }

    #[test]
    fn cancelling_reports_cancelled() {
        let cancel = CancelToken::new();
        cancel.cancel();
        let err = generator("return function() while true do end end").generate(&request(ParamValues::new()), &cancel);
        assert_eq!(err.unwrap_err(), TerrainError::Cancelled);
    }
}
