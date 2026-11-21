# Writing plugins

A plugin is a folder. Psito 3D looks for plugin folders in, in order:

1. `~/.local/share/psito-3d/plugins/` (Windows: `%APPDATA%\psito-3d\plugins\`)
2. `plugins/` next to the program
3. `/usr/share/psito-3d/plugins/` when installed

When two plugins share an id, the first one found wins, so your own folder
can replace a bundled plugin.

## The manifest

`plugin.toml` says what the plugin provides. It is read **without running
any code**, so the editor can show the controls first.

```toml
id = "org.example.volcano"          # reverse domain name; scenes store it
name = { en = "Volcano", pt = "Vulcão" }
description = { en = "A cone with a crater.", pt = "Um cone com cratera." }
version = "1.0.0"
author = "You"
api = 1                             # the plugin API version you wrote for

[terrain]                           # this plugin makes terrains
script = "terrain.lua"

[[terrain.params]]
id = "crater"                       # the key your script reads
label = { en = "Crater", pt = "Cratera" }
kind = "slider"                     # slider | toggle | choice
min = 0.0
max = 0.4
default = 0.12

[[terrain.params]]
id = "gullies"
label = "Gullies"                   # a plain string counts as English
kind = "toggle"
default = true

[[terrain.params]]
id = "style"
label = "Style"
kind = "choice"
default = "round"
options = [
  { id = "round", label = "Round" },
  { id = "sharp", label = "Sharp" },
]
```

Unknown keys are an error, so typos are caught. A plugin may also bring a
theme pack: `theme = "theme.toml"` (see [THEMES.md](THEMES.md)).

## The script

The script returns a table with a `height` function, or the function itself.
It is called once per sample:

```lua
return {
  height = function(x, y, p)
    -- x and y go from -1 to 1 across the terrain.
    -- p holds the parameters: p.crater, p.gullies, p.style.
    -- Return a height from 0 (the floor) to 1 (the top).
    return 0.5
  end,
}
```

The border of the terrain is sloped down to the floor afterwards (the
"Coast" setting), so you do not need to.

## The `psito` API (version 1)

| Function | Returns |
|---|---|
| `psito.api` | `1` |
| `psito.seed` | the scene's seed |
| `psito.noise.perlin(x, y [, seed])` | gradient noise, about -1 to 1, one feature per unit |
| `psito.noise.fbm(x, y [, opts])` | layered noise, about -1 to 1: rolling hills |
| `psito.noise.ridged(x, y [, opts])` | ridged noise, 0 to 1: mountain crests |
| `psito.math.smoothstep(a, b, x)` | 0 below `a`, 1 above `b`, smooth between |
| `psito.math.clamp(x, lo, hi)` | `x` held between `lo` and `hi` |
| `psito.math.mix(a, b, t)` | `a` to `b` as `t` goes from 0 to 1 |

`opts` is a table with any of `octaves` (1 to 16, default 6), `lacunarity`
(default 2), `gain` (default 0.5) and `seed` (mixed with the scene's seed).

The noise is the same Rust code the built-in terrains use: fast, and the
same on every computer. `math.random` works and is seeded from the scene, so
the same scene always gives the same terrain.

## The sandbox

Plugins can do exactly what the API above allows, and no more:

- only the `math`, `string`, `table` and `utf8` libraries; no `io`, `os`,
  `package`, `debug` or `coroutine`;
- no `load`, `loadfile`, `dofile`, `require`, `string.dump` or
  `collectgarbage`; scripts are loaded as text, never as bytecode;
- `print` writes to the program's log, marked with your plugin id;
- 64 MB of memory and 30 seconds per terrain; a script over either is stopped
  and its error shown in the status bar;
- files named in the manifest must be inside the plugin folder (links that
  lead out are refused).

## Errors

A plugin that does not load is listed in the status bar (hover it for the
reasons). A script error shows the file and line:
`The terrain failed: terrain.lua:12: attempt to index a nil value`.
