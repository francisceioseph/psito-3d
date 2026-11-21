# Psito 3D

Make landscapes the way it felt in the 90s: drag the sun across the sky, roll
a new world with a die, melt chrome balls into each other and watch the
picture sharpen while you look at it. Terrains and themes can come from
plugins, written in Lua.

Psito 3D is a sibling of [Psito Plume](https://github.com/francisceioseph/psito-plume)
and [Psito Player](https://github.com/francisceioseph/psito-player): a program for
everyone, on Linux, Windows and macOS, with no account, no ads and no
telemetry. It is inspired by the landscape programs of the 90s, with its own
interface.

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
- **Use as wallpaper**, rendered at the size of your screen, on GNOME,
  Cinnamon, MATE, KDE Plasma, Xfce, Windows and macOS
- **Themes** that restyle every control, built in or from plugins
- In English and Portuguese

## Install

### On Windows

Windows 10 version 1809 or newer, or Windows 11, on x64. Download
[the installer](https://github.com/francisceioseph/psito-3d/releases/latest/download/psito-3d-windows-x64-setup.exe)
and run it. It doesn't ask for administrator rights.

The installer isn't signed with a paid certificate, so SmartScreen may warn that it
comes from an unknown publisher: choose **More info**, then **Run anyway**.

### From the APT repository

On Debian 13 or newer, Ubuntu 24.04 or newer, or Mint 22 or newer. It updates along with the system:

```bash
sudo mkdir -p /etc/apt/keyrings
curl -fsSL https://francisceioseph.github.io/psito-3d/psito-3d.asc | sudo tee /etc/apt/keyrings/psito-3d.asc >/dev/null
printf 'Types: deb\nURIs: https://francisceioseph.github.io/psito-3d\nSuites: stable\nComponents: main\nArchitectures: amd64\nSigned-By: /etc/apt/keyrings/psito-3d.asc\n' | sudo tee /etc/apt/sources.list.d/psito-3d.sources >/dev/null
sudo apt update
sudo apt install psito-3d
```

### From the DNF repository

On Fedora, on x86_64. The first install asks you to accept the signing key:

```bash
sudo curl -fsSL -o /etc/yum.repos.d/psito-3d.repo https://francisceioseph.github.io/psito-3d/psito-3d.repo
sudo dnf install psito-3d
```

### From the source

On any system with Rust 1.88 or newer and a GPU with Vulkan, Metal or DirectX 12
(Mesa's llvmpipe works, slowly). This is also the way on macOS, for now.

```bash
cargo run --release
```

## Use

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
tool/build-deb.sh      # target/packages/psito-3d_<version>_amd64.deb
tool/build-rpm.sh      # the same as .rpm (needs rpmbuild)
```

Releases are made by the CI: see [docs/RELEASING.md](docs/RELEASING.md).

How the code is organised, and why, is in
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## License

Apache 2.0.
