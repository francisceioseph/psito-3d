# Releasing

Releases are automatic. Every push to `main` runs the CI; when the checks
pass, [semantic-release](https://semantic-release.gitbook.io/) reads the
commit messages since the last tag and, if there is something to release,
picks the version (`feat` raises the minor, `fix` the patch, a
`BREAKING CHANGE` the major), writes it with `tool/set-version.sh` (into
`Cargo.toml`, `Cargo.lock` and the AppStream release list), tags it and
publishes the GitHub release. Then:

- `packages.yml` builds the `.deb` and the `.rpm`, attaches them to the
  release, and publishes on GitHub Pages the project page (`site/`) with
  the signed APT and DNF repositories of the last three versions;
- `windows.yml` builds the Inno Setup installer and attaches it.

## What the packages contain

| File | Linux | Windows |
|---|---|---|
| The program | `/usr/bin/psito-3d` | `psito-3d.exe` (icon and version embedded by `build.rs`) |
| Bundled plugins | `/usr/share/psito-3d/plugins/` | `plugins\` next to the program |
| Launcher | `org.psito.psito3d.desktop` | Start menu (and desktop, if chosen) |
| Software centre data | `org.psito.psito3d.metainfo.xml` | — |
| Icon | hicolor 48 to 512 px | inside the `.exe` |
| Licenses | `/usr/share/doc/psito-3d/` | `licenses\` |

The program finds the bundled plugins next to itself or in
`../share/psito-3d/plugins`, so the same binary works in every layout.

The binary links only to the C library: Vulkan, X11, Wayland and
xkbcommon are opened at run time, so the package dependencies are written by
hand in `tool/build-deb.sh` and `tool/build-rpm.sh`. When a dependency is
added, check them by installing the package in a clean container and
starting the program under `xvfb-run` with `mesa-vulkan-drivers`: the log
must say how many plugins loaded and nothing must panic.

The third-party licenses are generated with
[cargo-about](https://github.com/EmbarkStudios/cargo-about) from
`about.toml` and `about.hbs`; a dependency under a license not listed in
`about.toml` fails the CI.

## The icon

`assets/` holds the icon, rendered by Psito 3D itself:

```bash
cargo run --release -p psito3d-render --example icon -- assets
```

## Once, before the first release

1. Create the GitHub repository `francisceioseph/psito-3d` and push `main`.
2. Create the signing key of the repositories: `tool/create-apt-key.sh`
   (it stores it as the `APT_SIGNING_KEY` secret). Keep the backup it writes
   somewhere safe: a new key would make everyone who installed add it again.
3. In the repository settings, set GitHub Pages to deploy from GitHub Actions.
