# Psito 3D

Make landscapes the way it felt in the 90s: drag the sun across the sky, roll
a new world with a die, melt chrome balls into each other and watch the
picture sharpen while you look at it. Terrains and themes can come from
plugins, written in Lua.

Psito 3D is a sibling of [Psito Plume](https://github.com/francisceioseph/psito-plume)
and [Psito Player](https://github.com/francisceioseph/psito-player), and the 3D
toy of [Psito OS](https://github.com/francisceioseph/psito-os). It is inspired by
the landscape programs of the 90s, with its own interface.

## What it does today

- **Sun you can drag** on a dial of the sky, from noon to sunset to night
- **Surprise me**: a whole new world from one seed
- **Terrains** from three built-in shapes (hills, island, mountains), from
  Lua plugins, or **from a picture**, where white is high and black is low
- **The endless sea**, a mirror up to the horizon, with ripples
- **Fog and valley mist**, haze and clouds
- **Objects**: balls, boxes, cylinders, cones and rings in chrome, gold,
  glass or clay, that can be added, carved out or **melted together**
- **Progressive rendering** on the GPU: a quick preview while you drag, then
  a clean image that sharpens sample by sample
- **Finishing filters**: 256 colours with dithering, or a tube screen
- **Undo and redo**, scenes saved as `.psito3d`, pictures saved as PNG
- **Themes** that restyle every control, built in or from plugins
- In English and Portuguese

## Run

You need Rust 1.88 or newer and a GPU with Vulkan, Metal or DirectX 12
(Mesa's llvmpipe works, slowly).

```bash
cargo run --release
```

Drag on the image to turn around, right-drag (or Shift-drag) to slide, and
scroll to come closer. Ctrl+Z and Ctrl+Shift+Z undo and redo, Ctrl+S saves,
Ctrl+O opens and Ctrl+E saves a picture.

## Plugins

A plugin is a folder with a `plugin.toml` and a Lua script. This one is a
whole terrain generator:

```lua
return {
  height = function(x, y, p)
    local d = math.sqrt(x * x + y * y)
    return psito.math.clamp((1 - d) * p.height + 0.1 * psito.noise.fbm(x * 4, y * 4), 0, 1)
  end,
}
```

Plugins run in a sandbox: no files, no network, no clock, a memory limit and a
time limit. See [docs/PLUGINS.md](docs/PLUGINS.md) and the examples in
[plugins/](plugins/). Themes are described in [docs/THEMES.md](docs/THEMES.md).

## Develop

```bash
tool/check.sh          # format, lint and every test
tool/check.sh --quick  # without the GPU and end-to-end tests
cargo run --release -p psito3d-render --example snapshot -- out.png
```

How the code is organised, and why, is in
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## License

Apache 2.0.
