//! Putting a picture on the desktop, on whatever desktop the person uses.
//!
//! Each desktop has its own way, so this is split in two:
//!
//! - [`detect`] and [`plan`] are pure: they decide *which* desktop this is
//!   and *what* commands would set the wallpaper there. They are tested for
//!   every desktop without touching anyone's real wallpaper.
//! - [`SystemWallpaper`] runs those commands. The app receives a
//!   [`WallpaperSetter`] (the Strategy pattern), so the end-to-end tests
//!   give it a fake one.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The desktops that can be told to change the wallpaper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Desktop {
    /// GNOME and the desktops built on its settings (Ubuntu, Budgie, Pantheon, Unity).
    Gnome,
    /// Cinnamon (Linux Mint).
    Cinnamon,
    /// MATE.
    Mate,
    /// KDE Plasma.
    Kde,
    /// Xfce.
    Xfce,
    /// Windows.
    Windows,
    /// macOS.
    MacOs,
}

/// A command to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cmd {
    /// The program.
    pub program: String,
    /// Its arguments.
    pub args: Vec<String>,
    /// Whether a failure is fine (a setting older versions lack).
    pub optional: bool,
}

impl Cmd {
    fn new(program: &str, args: &[&str]) -> Self {
        Self { program: program.to_owned(), args: args.iter().map(|a| (*a).to_owned()).collect(), optional: false }
    }

    fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

/// What can go wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WallpaperError {
    /// This desktop has no known way to set the wallpaper.
    Unsupported,
    /// A command failed; the text says which and why.
    Failed(String),
}

impl std::fmt::Display for WallpaperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => f.write_str("unsupported desktop"),
            Self::Failed(why) => f.write_str(why),
        }
    }
}

/// The desktop, from the operating system (`std::env::consts::OS`) and, on
/// Linux and the BSDs, `XDG_CURRENT_DESKTOP` (such as `ubuntu:GNOME`).
pub fn detect(os: &str, xdg_current_desktop: Option<&str>) -> Option<Desktop> {
    match os {
        "windows" => return Some(Desktop::Windows),
        "macos" => return Some(Desktop::MacOs),
        _ => {}
    }
    // The list is ordered from the most specific name to the most general,
    // and Cinnamon and Budgie say GNOME too.
    let names: Vec<String> = xdg_current_desktop?.split(':').map(|n| n.trim().to_ascii_lowercase()).collect();
    let has = |n: &str| names.iter().any(|x| x == n);
    if has("x-cinnamon") || has("cinnamon") {
        Some(Desktop::Cinnamon)
    } else if has("mate") {
        Some(Desktop::Mate)
    } else if has("kde") {
        Some(Desktop::Kde)
    } else if has("xfce") {
        Some(Desktop::Xfce)
    } else if ["gnome", "unity", "budgie", "pantheon", "gnome-flashback"].iter().any(|n| has(n)) {
        Some(Desktop::Gnome)
    } else {
        None
    }
}

fn file_uri(path: &Path) -> String {
    // Percent-encode what a URI cannot hold; letters, digits and the path
    // punctuation stay as they are.
    let mut uri = String::from("file://");
    for b in path.to_string_lossy().bytes() {
        if b.is_ascii_alphanumeric() || b"/-_.~".contains(&b) {
            uri.push(b as char);
        } else {
            uri.push_str(&format!("%{b:02X}"));
        }
    }
    uri
}

/// The commands that set `path` as the wallpaper on `desktop`. Xfce keeps
/// one setting per monitor and workspace: `xfce_props` are the property
/// names `xfconf-query -c xfce4-desktop -l` listed.
pub fn plan(desktop: Desktop, path: &Path, xfce_props: &[String]) -> Vec<Cmd> {
    let p = path.to_string_lossy().into_owned();
    let uri = file_uri(path);
    let gsettings = |schema: &str, key: &str, value: &str| Cmd::new("gsettings", &["set", schema, key, value]);
    match desktop {
        Desktop::Gnome => vec![
            gsettings("org.gnome.desktop.background", "picture-uri", &uri),
            // Only GNOME 42 and newer have a picture for the dark style.
            gsettings("org.gnome.desktop.background", "picture-uri-dark", &uri).optional(),
            gsettings("org.gnome.desktop.background", "picture-options", "zoom").optional(),
        ],
        Desktop::Cinnamon => vec![
            gsettings("org.cinnamon.desktop.background", "picture-uri", &uri),
            gsettings("org.cinnamon.desktop.background", "picture-options", "zoom").optional(),
        ],
        Desktop::Mate => vec![
            gsettings("org.mate.background", "picture-filename", &p),
            gsettings("org.mate.background", "picture-options", "zoom").optional(),
        ],
        Desktop::Kde => vec![Cmd::new("plasma-apply-wallpaperimage", &[&p])],
        Desktop::Xfce => xfce_props
            .iter()
            .filter(|prop| prop.ends_with("/last-image"))
            .map(|prop| Cmd::new("xfconf-query", &["-c", "xfce4-desktop", "-p", prop, "-s", &p]))
            .collect(),
        Desktop::MacOs => {
            let script = format!(
                "tell application \"System Events\" to tell every desktop to set picture to \"{}\"",
                p.replace('\\', "\\\\").replace('"', "\\\"")
            );
            vec![Cmd::new("osascript", &["-e", &script])]
        }
        Desktop::Windows => {
            // SystemParametersInfo through PowerShell, which keeps unsafe
            // code out of the program. 20 is SPI_SETDESKWALLPAPER, and 3
            // writes the setting and tells every window.
            let script = format!(
                "Add-Type -TypeDefinition 'using System.Runtime.InteropServices; public class W {{ \
                 [DllImport(\"user32.dll\", CharSet = CharSet.Unicode)] \
                 public static extern int SystemParametersInfo(int a, int b, string c, int d); }}'; \
                 if ([W]::SystemParametersInfo(20, 0, '{}', 3) -eq 0) {{ exit 1 }}",
                p.replace('\'', "''")
            );
            vec![Cmd::new("powershell", &["-NoProfile", "-NonInteractive", "-Command", &script])]
        }
    }
}

/// Something that can put a picture on the desktop.
pub trait WallpaperSetter: Send + Sync + std::fmt::Debug {
    /// Makes `path` the wallpaper.
    fn set(&self, path: &Path) -> Result<(), WallpaperError>;
}

/// The real one: finds the desktop and runs the commands.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemWallpaper;

fn run(cmd: &Cmd) -> Result<String, WallpaperError> {
    let mut command = Command::new(&cmd.program);
    command.args(&cmd.args);
    #[cfg(windows)]
    {
        // No console window flashing behind the program.
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    let out = command.output().map_err(|e| WallpaperError::Failed(format!("{}: {e}", cmd.program)))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        Err(WallpaperError::Failed(format!("{}: {}", cmd.program, err.lines().next().unwrap_or("failed"))))
    }
}

impl WallpaperSetter for SystemWallpaper {
    fn set(&self, path: &Path) -> Result<(), WallpaperError> {
        let desktop = detect(std::env::consts::OS, std::env::var("XDG_CURRENT_DESKTOP").ok().as_deref())
            .ok_or(WallpaperError::Unsupported)?;
        let props = if desktop == Desktop::Xfce {
            run(&Cmd::new("xfconf-query", &["-c", "xfce4-desktop", "-l"]))?.lines().map(str::to_owned).collect()
        } else {
            Vec::new()
        };
        let cmds = plan(desktop, path, &props);
        if cmds.is_empty() {
            return Err(WallpaperError::Unsupported);
        }
        for cmd in &cmds {
            match run(cmd) {
                Err(e) if !cmd.optional => return Err(e),
                Err(e) => log::info!("optional wallpaper setting skipped: {e}"),
                Ok(_) => {}
            }
        }
        log::info!("wallpaper set on {desktop:?}: {}", path.display());
        Ok(())
    }
}

/// A new file name for a wallpaper in `dir`. Each wallpaper gets its own
/// name: desktops do not reload a picture whose path did not change.
pub fn new_wallpaper_path(dir: &Path, stamp: u64) -> PathBuf {
    dir.join(format!("psito-3d-{stamp}.png"))
}

/// Removes all but the `keep` newest wallpapers this program made in `dir`.
pub fn prune_wallpapers(dir: &Path, keep: usize) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut ours: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("psito-3d-") && n.ends_with(".png"))
        })
        .collect();
    // The names carry the time, so sorting them sorts by age.
    ours.sort_by_key(|p| {
        p.file_stem().and_then(|s| s.to_str()).and_then(|s| s.trim_start_matches("psito-3d-").parse::<u64>().ok())
    });
    let excess = ours.len().saturating_sub(keep);
    for old in &ours[..excess] {
        if let Err(e) = std::fs::remove_file(old) {
            log::warn!("could not remove the old wallpaper {}: {e}", old.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_each_desktop() {
        assert_eq!(detect("linux", Some("X-Cinnamon")), Some(Desktop::Cinnamon));
        assert_eq!(detect("linux", Some("ubuntu:GNOME")), Some(Desktop::Gnome));
        assert_eq!(detect("linux", Some("Budgie:GNOME")), Some(Desktop::Gnome));
        assert_eq!(detect("linux", Some("KDE")), Some(Desktop::Kde));
        assert_eq!(detect("linux", Some("XFCE")), Some(Desktop::Xfce));
        assert_eq!(detect("linux", Some("MATE")), Some(Desktop::Mate));
        assert_eq!(detect("windows", None), Some(Desktop::Windows));
        assert_eq!(detect("macos", None), Some(Desktop::MacOs));
    }

    #[test]
    fn unknown_desktops_are_unsupported() {
        assert_eq!(detect("linux", Some("sway")), None);
        assert_eq!(detect("linux", Some("Psito:labwc")), None);
        assert_eq!(detect("linux", None), None);
    }

    #[test]
    fn gnome_sets_both_styles_and_encodes_the_uri() {
        let cmds = plan(Desktop::Gnome, Path::new("/home/ana/My Pictures/a.png"), &[]);
        assert_eq!(
            cmds[0].args,
            ["set", "org.gnome.desktop.background", "picture-uri", "file:///home/ana/My%20Pictures/a.png"]
        );
        assert_eq!(cmds[1].args[2], "picture-uri-dark");
        assert!(!cmds[0].optional && cmds[1].optional);
    }

    #[test]
    fn cinnamon_and_mate_use_their_own_schemas() {
        let c = plan(Desktop::Cinnamon, Path::new("/w.png"), &[]);
        assert_eq!(c[0].args[1], "org.cinnamon.desktop.background");
        let m = plan(Desktop::Mate, Path::new("/w.png"), &[]);
        assert_eq!(m[0].args, ["set", "org.mate.background", "picture-filename", "/w.png"]);
    }

    #[test]
    fn xfce_sets_every_monitor_and_workspace() {
        let props: Vec<String> = [
            "/backdrop/screen0/monitorHDMI-1/workspace0/last-image",
            "/backdrop/screen0/monitorHDMI-1/workspace0/image-style",
            "/backdrop/screen0/monitoreDP-1/workspace0/last-image",
        ]
        .map(str::to_owned)
        .to_vec();
        let cmds = plan(Desktop::Xfce, Path::new("/w.png"), &props);
        assert_eq!(cmds.len(), 2);
        assert!(cmds.iter().all(|c| c.args.last().map(String::as_str) == Some("/w.png")));
        assert!(plan(Desktop::Xfce, Path::new("/w.png"), &[]).is_empty());
    }

    #[test]
    fn quotes_in_paths_cannot_break_the_scripts() {
        let mac = plan(Desktop::MacOs, Path::new("/Users/a\"b/w.png"), &[]);
        assert!(mac[0].args[1].contains(r#"a\"b"#));
        let win = plan(Desktop::Windows, Path::new(r"C:\Users\O'Neil\w.png"), &[]);
        assert!(win[0].args[3].contains("O''Neil"));
    }

    #[test]
    fn only_the_newest_wallpapers_are_kept() {
        let dir = tempfile::tempdir().unwrap();
        for stamp in [5, 30, 7, 100] {
            std::fs::write(new_wallpaper_path(dir.path(), stamp), b"png").unwrap();
        }
        std::fs::write(dir.path().join("mine.png"), b"keep me").unwrap();
        prune_wallpapers(dir.path(), 2);
        let mut left: Vec<String> =
            std::fs::read_dir(dir.path()).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
        left.sort();
        assert_eq!(left, ["mine.png", "psito-3d-100.png", "psito-3d-30.png"]);
    }
}
