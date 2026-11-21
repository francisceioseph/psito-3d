app-title = Psito 3D

## Toolbar
surprise-me = Surprise me
new-scene = New scene
open-scene = Open scene
save-scene = Save scene
export-image = Save picture
set-wallpaper = Use as wallpaper
undo = Undo
redo = Redo
filter-none = Clean
filter-dither = 256 colours
theme = Theme
language = Language
filter-crt = Tube screen

## Scene code
scene-code = Scene code
scene-code-share = Copy this code to share this world: whoever opens it gets the same scene.
scene-code-picture = This terrain is a picture on this computer; others will need the same picture.
scene-code-paste = Paste a code to open its world
scene-code-hint = A code starts with p3d1.
copy-code = Copy code
code-copied = Copied
open-code = Open
close = Close
code-invalid = This is not a Psito 3D scene code, or part of it is missing.
code-too-new = This code was made by a newer Psito 3D.
orbit = Turn around by itself

## Sky
section-sky = Sky
sun = Sun
sun-height = Sun height
sun-direction = Sun direction
haze = Haze
clouds = Clouds
cloud-height = Cloud height
exposure = Brightness

## Terrain
section-terrain = Terrain
generator = Shape
generator-missing = This shape comes from a plugin that is not installed.
source-picture = From a picture
choose-picture = Choose picture
seed = Seed
roll-seed = Roll a new seed
terrain-height = Vertical scale
coast = Coast
resolution = Detail

## Brushes
section-brushes = Brushes
brush-off = Off
brush-raise = Raise
brush-lower = Dig
brush-smooth = Smooth
brush-flatten = Flatten
brush-size = Brush size
brush-strength = Strength
brush-hint = Drag on the image to paint the ground. The right button turns around, the middle one slides.
clear-strokes = Clear strokes
brush-strokes = { $count ->
    [0] No strokes
    [one] One stroke
   *[other] { $count } strokes
}

## Sea and fog
section-sea = Sea
sea = Sea
sea-color = Water colour
sea-level = Water level
ripple = Ripples
section-fog = Fog
fog = Fog
mist = Mist in the valleys
mist-height = Mist height

## Objects
section-objects = Objects
add-sphere = Add a ball
add-cube = Add a box
add-cylinder = Add a cylinder
add-cone = Add a cone
add-torus = Add a ring
shape-sphere = Ball
shape-cube = Box
shape-cylinder = Cylinder
shape-cone = Cone
shape-torus = Ring
no-objects = No objects yet. Add one with the buttons above.
show-object = Show
hide-object = Hide
object-name = Name
material = Material
material-chrome = Chrome
material-gold = Gold
material-glass = Glass
material-clay = Clay
color = Colour
roughness = Roughness
metallic = Metal
transparency = Transparency
combine = Joins the others by
combine-union = Adding
combine-subtract = Carving
combine-intersect = Overlapping
blend = Melting
position-x = Left and right
position-y = Up and down
position-z = Near and far
size = Size
turn = Turn
tilt = Tilt
delete-object = Delete object

## Status
status-samples = Sharpening: { $done } of { $total }
status-done = Finished
terrain-working = Building the terrain
terrain-failed = The terrain failed: { $reason }
plugin-errors = { $count ->
    [one] One plugin did not load
   *[other] { $count } plugins did not load
}
wallpaper-working = Making the wallpaper
wallpaper-set = Wallpaper changed
wallpaper-unsupported = this desktop has no known way to change it
wallpaper-failed = Could not change the wallpaper ({ $reason }). The picture is in { $path }
saved = Saved
exported = Picture saved
open-failed = Could not open: { $reason }
save-failed = Could not save: { $reason }

## Viewport
viewport = Rendered scene
