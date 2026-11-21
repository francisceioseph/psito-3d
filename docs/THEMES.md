# Theme packs

A theme pack is a TOML file that restyles every control. Two ship with the
program (`crates/ui/themes/`); plugins can bring more with
`theme = "theme.toml"` in their manifest (see `plugins/theme-vaporwave/`).

```toml
id = "org.example.sunset"           # unique; the preferences store it
name = { en = "Sunset", pt = "Pôr do sol" }
dark = true                         # for egui's own widgets

[colors]                            # "#rrggbb" or "#rrggbbaa"
background = "#14172a"              # behind everything
panel = "#1e2238"                   # panels and cards
raised = "#2c3150"                  # buttons, the slider thumb
sunken = "#121527"                  # slider grooves, text fields
text = "#eceef7"
text_muted = "#9aa1c2"              # captions
accent = "#ff8a4c"                  # active controls, the selection, the sun
accent_text = "#1a1205"             # text on the accent
line = "#383e62"                    # borders
bevel_light = "#ffffff26"           # the lit edge of a bevel
bevel_dark = "#00000080"            # the shaded edge
danger = "#ff6b6b"                  # errors

[metrics]                           # optional; these are the defaults
radius = 5.0                        # corner radius, 0 to 30
bevel = 1.5                         # bevel width, 0 (flat) to 6
stroke = 1.0                        # line width, 0 to 6
spacing = 8.0                       # base spacing, 2 to 24
control_height = 26.0               # 16 to 48

[type]                              # optional font sizes
body = 14.0
heading = 15.0
small = 12.0

[components]                        # optional component styles
button = "bevel"                    # bevel | flat | pill
slider = "groove"                   # groove | line
```

Every colour key is required, so a pack never leaves a control unstyled.
Unknown keys and sizes out of range are refused, and the error is shown in
the status bar.
