# Architecture

Psito 3D is a Cargo workspace of six crates. Each one has a single job and
depends only on the ones below it:

```
psito-3d (app)       eframe window, pages, files, wiring
 ├── psito3d-ui      UI kit: theme tokens, atoms → molecules → organisms → templates
 ├── psito3d-render  wgpu ray marcher, progressive accumulation
 ├── psito3d-plugins Lua host and sandbox
 │    └── psito3d-terrain   heightfields, noise, terrain generators
 └── psito3d-scene   the scene model, file format, undo history
```

`psito3d-scene` and `psito3d-terrain` are plain Rust with no GPU or window,
so most of the logic is tested in milliseconds.

## The scene

A `Scene` is data: terrain recipe (with the brush strokes), sky, fog, sea,
objects, camera, filter. It never says how it is drawn. The camera is the
view, not the work: moving it is never an undo step (`Scene::same_world`). Values read from files are clamped
(`Scene::sanitize`, `ParamSpec::sanitize`) because files and plugins can be
edited by hand. The file format (`.psito3d`) is JSON with a `format` version;
newer files are refused instead of being read wrong.

## Patterns, and why

| Pattern | Where | Why |
|---|---|---|
| Strategy | `TerrainGenerator` (built-ins and `LuaTerrainGenerator`) | The editor talks to one trait; a plugin generator is as good as a built-in one. |
| Memento | `History` (snapshots of the scene) | A scene is a few KB. Snapshots cannot get out of step with the scene, as a list of commands can. One drag is one step, because the editor settles changes only when the mouse is up (`EditorState::settle`). |
| Registry | `PluginRegistry`, `Catalog` | Plugins found on disk, checked once, looked up by id. |
| Atomic design | `psito3d-ui` | Small widgets compose into panels; a theme changes all of them at once. |
| Design tokens | `Theme` | No widget has a literal colour or size; they read the theme from the egui context. |
| Dependency injection | `AppOptions`, `EditorPage` | Paths, seeds and data come in from outside, so tests can run the real app on temporary folders. |
| Strategy, again | `WallpaperSetter` (`SystemWallpaper`, a fake in the tests) | Each desktop sets the wallpaper its own way; `detect` and `plan` decide what to run and are tested for every desktop, and the end-to-end tests never touch the real wallpaper. |
| Worker with cancellation | `TerrainWorker`, `CancelToken` | Dragging a slider replaces the terrain being built instead of queueing them. The worker keeps the generator's ground, so painting only redraws the strokes. |
| Recipe, not result | `TerrainSettings::strokes` | Brushes store their strokes, not the changed heights: the scene stays small (and fits in a scene code), the world rebuilds the same from its recipe, and one stroke is one undo step. |

## Rendering

`psito3d-render` ray marches the scene in a WGSL fragment shader
(`shaders/trace.wgsl`): the terrain is a heightfield texture, objects are
signed distance functions combined in order (union, subtraction,
intersection, each optionally smooth), the sea is a plane, the sky is
analytic. Paths follow mirror, glass and water reflections for a few bounces.

Each frame adds **one sample per pixel** to a 32-bit float accumulation
texture (two textures used in turn), with a sub-pixel jitter from the Halton
sequence. `resolve.wgsl` tone maps (ACES) into an sRGB texture and applies
the finishing filter. While something changes the renderer draws half-size
previews (`Quality::Preview`); when it stops it accumulates the clean image
and then goes idle.

The CPU side packs the scene into `#[repr(C)]` structs of `vec4`s
(`gpu.rs`), so the WGSL layout needs no padding tricks; a test checks sizes.

## UI kit

`psito3d-ui` follows atomic design:

- **atoms**: `Button`, `IconButton`, `Slider`, `Toggle`, `Swatch`, vector `Icon`s;
- **molecules**: `LabeledSlider`, `SunDial`, `SeedField`, `ChoiceChips`, `section`;
- **organisms**: the sky, terrain, sea and fog, and objects panels, the toolbar,
  the status bar, the viewport (camera controls), the parameter form;
- **templates**: `EditorLayout` (where things go, not what they are).

The page (`EditorPage`, in the app) fills the template with real data.
Organisms edit the scene in place and return only what they cannot do
themselves (open a dialog, roll a seed).

Every control has an accessible name. The UI tests find controls by those
names, so a control without one cannot be tested, and egui_kittest also fails
any test where a widget lacks one.

Texts are Fluent files (`crates/ui/locales/*.ftl`); a test keeps the languages
in step. Plugin and theme texts are translated in their own manifests.

## Plugins

See [PLUGINS.md](PLUGINS.md). The host reads the manifest without running
code, then runs scripts in a fresh sandboxed Lua 5.4 per job (so a run never
leaks into the next). The noise functions plugins call are the same Rust code
the built-ins use.

## Tests: the pyramid

| Level | What | Where | Count |
|---|---|---|---|
| Unit | Model, math, noise, sandbox, parsing, packing, undo | `#[cfg(test)]` in each crate | most |
| Integration | Crates through their public API: file format, generators, plugin folders, shader validation (naga), real GPU renders | `crates/*/tests/` | fewer |
| UI | Widgets and panels driven by name with egui_kittest | `crates/ui/tests/` | fewer |
| End to end | The real app, real renderer, real plugins, driven through its UI | `crates/app/tests/e2e.rs` | few |

GPU tests run on any adapter (llvmpipe included) and skip when there is none,
unless `PSITO3D_REQUIRE_GPU=1` (the CI sets it). Set `PSITO3D_SNAPSHOT_DIR`
to keep the end-to-end screenshots.
