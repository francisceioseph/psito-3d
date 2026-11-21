//! The sandbox every plugin runs in.
//!
//! What is removed, and why:
//!
//! - the `io`, `os`, `package`, `debug` and `coroutine` libraries are never
//!   loaded: no files, no processes, no clock, no native modules;
//! - `load`, `loadfile`, `dofile` and `require` are removed, and chunks are
//!   loaded as text only, so no bytecode (which can crash the VM) gets in;
//! - `string.dump` and `collectgarbage` are removed;
//! - `print` writes to the program log, marked with the plugin id;
//! - `math.random` is seeded from the scene, so a plugin stays deterministic.
//!
//! On top of that a memory limit, and a hook every few thousand instructions
//! that stops the script when it runs out of time or is cancelled.

use std::cell::RefCell;
use std::time::{Duration, Instant};

use mlua::chunk::ChunkMode;
use mlua::{HookTriggers, Lua, LuaOptions, StdLib, Value, VmState};
use psito3d_scene::SplitMix64;
use psito3d_terrain::CancelToken;

/// How much a plugin may use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Memory, in bytes.
    pub memory: usize,
    /// Time for one whole job, such as making a terrain.
    pub time: Duration,
}

impl Default for Limits {
    fn default() -> Self {
        Self { memory: 64 * 1024 * 1024, time: Duration::from_secs(30) }
    }
}

/// The message of the error raised when time runs out.
pub const TIME_LIMIT_MESSAGE: &str = "the plugin ran out of time";
/// The message of the error raised when the job is cancelled.
pub const CANCELLED_MESSAGE: &str = "cancelled";

/// A fresh Lua state with the sandbox in place.
///
/// `log_name` marks the plugin's `print` lines in the log, and `seed` seeds
/// `math.random`.
pub fn new_sandbox(limits: Limits, cancel: CancelToken, log_name: &str, seed: u64) -> mlua::Result<Lua> {
    let lua = Lua::new_with(StdLib::MATH | StdLib::STRING | StdLib::TABLE | StdLib::UTF8, LuaOptions::default())?;
    let globals = lua.globals();
    for name in ["dofile", "loadfile", "load", "require", "collectgarbage"] {
        globals.set(name, Value::Nil)?;
    }
    let string: mlua::Table = globals.get("string")?;
    string.set("dump", Value::Nil)?;

    let tag = log_name.to_owned();
    globals.set(
        "print",
        lua.create_function(move |_, args: mlua::Variadic<Value>| {
            let parts: Vec<String> = args.iter().map(|v| v.to_string().unwrap_or_else(|_| "?".into())).collect();
            log::info!("[{tag}] {}", parts.join("\t"));
            Ok(())
        })?,
    )?;

    let rng = RefCell::new(SplitMix64::new(seed));
    let math: mlua::Table = globals.get("math")?;
    math.set(
        "random",
        lua.create_function(move |_, (m, n): (Option<i64>, Option<i64>)| {
            let mut rng = rng.borrow_mut();
            Ok(match (m, n) {
                (None, _) => Value::Number(rng.next_f32() as f64),
                (Some(m), None) if m >= 1 => Value::Integer(1 + (rng.next_u64() % m as u64) as i64),
                (Some(m), Some(n)) if n >= m => Value::Integer(m + (rng.next_u64() % (n - m + 1) as u64) as i64),
                _ => return Err(mlua::Error::runtime("bad argument to 'random' (interval is empty)")),
            })
        })?,
    )?;
    math.set("randomseed", Value::Nil)?;

    lua.set_memory_limit(limits.memory)?;
    let deadline = Instant::now() + limits.time;
    lua.set_hook(HookTriggers::new().every_nth_instruction(4096), move |_, _| {
        if cancel.is_cancelled() {
            Err(mlua::Error::runtime(CANCELLED_MESSAGE))
        } else if Instant::now() > deadline {
            Err(mlua::Error::runtime(TIME_LIMIT_MESSAGE))
        } else {
            Ok(VmState::Continue)
        }
    })?;
    Ok(lua)
}

/// Runs `source` as a text chunk named `name` and returns what it returns.
pub fn run_chunk(lua: &Lua, source: &str, name: &str) -> mlua::Result<Value> {
    lua.load(source).set_name(format!("@{name}")).set_mode(ChunkMode::Text).eval()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sandbox() -> Lua {
        new_sandbox(Limits::default(), CancelToken::new(), "test", 1).unwrap()
    }

    #[test]
    fn dangerous_things_are_gone() {
        let lua = sandbox();
        for expr in
            ["io", "os", "package", "debug", "load", "loadfile", "dofile", "require", "string.dump", "collectgarbage"]
        {
            let v: Value = run_chunk(&lua, &format!("return {expr}"), "t").unwrap();
            assert!(v.is_nil(), "{expr} is reachable");
        }
    }

    #[test]
    fn safe_libraries_work() {
        let lua = sandbox();
        let v: String = run_chunk(&lua, "return string.format('%d', math.floor(table.unpack({2.7})))", "t")
            .unwrap()
            .to_string()
            .unwrap();
        assert_eq!(v, "2");
    }

    #[test]
    fn bytecode_is_refused() {
        let lua = sandbox();
        let bytecode = "\x1bLua\x54\x00";
        assert!(run_chunk(&lua, bytecode, "t").is_err());
    }

    #[test]
    fn endless_loops_are_stopped() {
        let limits = Limits { time: Duration::from_millis(100), ..Default::default() };
        let lua = new_sandbox(limits, CancelToken::new(), "test", 1).unwrap();
        let err = run_chunk(&lua, "while true do end", "t").unwrap_err();
        assert!(err.to_string().contains(TIME_LIMIT_MESSAGE), "{err}");
    }

    #[test]
    fn cancel_stops_the_script() {
        let cancel = CancelToken::new();
        let lua = new_sandbox(Limits::default(), cancel.clone(), "test", 1).unwrap();
        cancel.cancel();
        let err = run_chunk(&lua, "while true do end", "t").unwrap_err();
        assert!(err.to_string().contains(CANCELLED_MESSAGE));
    }

    #[test]
    fn memory_is_limited() {
        let limits = Limits { memory: 4 * 1024 * 1024, ..Default::default() };
        let lua = new_sandbox(limits, CancelToken::new(), "test", 1).unwrap();
        let err = run_chunk(&lua, "local t = {} for i = 1, 1e8 do t[i] = i end", "t").unwrap_err();
        assert!(matches!(err, mlua::Error::MemoryError(_)), "{err}");
    }

    #[test]
    fn random_is_seeded() {
        let a: f64 = run_chunk(&sandbox(), "return math.random()", "t").unwrap().as_f64().unwrap();
        let b: f64 = run_chunk(&sandbox(), "return math.random()", "t").unwrap().as_f64().unwrap();
        assert_eq!(a, b);
        let d: i64 = run_chunk(&sandbox(), "return math.random(6)", "t").unwrap().as_i64().unwrap();
        assert!((1..=6).contains(&d));
    }
}
