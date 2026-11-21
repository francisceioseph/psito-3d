//! Brush strokes over a heightfield, and finding the ground under the mouse.

use psito3d_scene::{BrushStroke, BrushTool};

use crate::Heightfield;
use crate::noise::smoothstep;

/// How much one touch of a full-strength raise or lower moves the ground,
/// as a fraction of the terrain height. Points along a stroke overlap, so a
/// spot is touched several times per pass.
const STEP: f32 = 0.035;

/// Grid position of a world point, for a field of `n` samples per side.
fn to_grid(x: f32, z: f32, extent: f32, n: u32) -> (f32, f32) {
    let m = (n - 1) as f32;
    ((x / extent + 0.5) * m, (z / extent + 0.5) * m)
}

/// Paints `strokes`, in order, over `field`. `extent` is the side of the
/// terrain in world units. Deterministic: the same strokes always give the
/// same ground.
pub fn apply_strokes(field: &mut Heightfield, strokes: &[BrushStroke], extent: f32) {
    let n = field.size();
    for stroke in strokes {
        let Some(first) = stroke.points.first() else { continue };
        let (fx, fy) = to_grid(first[0], first[1], extent, n);
        // Flatten levels to where the stroke began, before it changed anything.
        let level = field.sample(fx / (n - 1) as f32 * 2.0 - 1.0, fy / (n - 1) as f32 * 2.0 - 1.0);
        let r = (stroke.radius / extent * (n - 1) as f32).max(0.75);
        for p in &stroke.points {
            let (cx, cy) = to_grid(p[0], p[1], extent, n);
            stamp(field, stroke.tool, stroke.strength, cx, cy, r, level);
        }
    }
}

fn stamp(field: &mut Heightfield, tool: BrushTool, strength: f32, cx: f32, cy: f32, r: f32, level: f32) {
    let n = field.size() as i64;
    let (x0, x1) = (((cx - r).floor() as i64).max(0), ((cx + r).ceil() as i64).min(n - 1));
    let (y0, y1) = (((cy - r).floor() as i64).max(0), ((cy + r).ceil() as i64).min(n - 1));
    for y in y0..=y1 {
        for x in x0..=x1 {
            let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt() / r;
            if d >= 1.0 {
                continue;
            }
            // A soft round brush: full in the middle, nothing at the rim.
            let k = 1.0 - smoothstep(0.0, 1.0, d);
            let h = field.get(x as u32, y as u32);
            let new = match tool {
                BrushTool::Raise => h + STEP * strength * k,
                BrushTool::Lower => h - STEP * strength * k,
                BrushTool::Smooth => {
                    let mut sum = 0.0;
                    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        sum += field.get((x + dx).clamp(0, n - 1) as u32, (y + dy).clamp(0, n - 1) as u32);
                    }
                    h + (sum * 0.25 - h) * (0.5 * strength * k)
                }
                BrushTool::Flatten => h + (level - h) * (0.3 * strength * k),
            };
            field.set(x as u32, y as u32, new);
        }
    }
}

/// Where a ray from `origin` along `dir` first meets the terrain, as world
/// `[x, y, z]`, or `None` if it misses. `extent` and `height` are the world
/// size of the terrain; `field` holds heights from 0 to 1.
pub fn raycast(field: &Heightfield, origin: [f32; 3], dir: [f32; 3], extent: f32, height: f32) -> Option<[f32; 3]> {
    let half = extent * 0.5;
    let ground = |x: f32, z: f32| {
        if x.abs() > half || z.abs() > half {
            return 0.0;
        }
        field.sample(x / half, z / half) * height
    };
    // Clip the ray to the box around the terrain (slab test).
    let (lo, hi) = ([-half, -0.01, -half], [half, height + 0.01, half]);
    let (mut t0, mut t1) = (0.0f32, f32::MAX);
    for a in 0..3 {
        if dir[a].abs() < 1e-8 {
            if origin[a] < lo[a] || origin[a] > hi[a] {
                return None;
            }
            continue;
        }
        let (ta, tb) = ((lo[a] - origin[a]) / dir[a], (hi[a] - origin[a]) / dir[a]);
        t0 = t0.max(ta.min(tb));
        t1 = t1.min(ta.max(tb));
    }
    if t0 > t1 {
        return None;
    }
    let at = |t: f32| [origin[0] + dir[0] * t, origin[1] + dir[1] * t, origin[2] + dir[2] * t];
    let above = |t: f32| {
        let p = at(t);
        p[1] - ground(p[0], p[2])
    };
    let step = extent / field.size() as f32 * 0.5;
    let mut prev = t0;
    let mut t = t0;
    while t <= t1 {
        if above(t) <= 0.0 {
            // Halve the last step a few times to land on the surface.
            let (mut a, mut b) = (prev, t);
            for _ in 0..20 {
                let m = (a + b) * 0.5;
                if above(m) > 0.0 { a = m } else { b = m }
            }
            return Some(at(b));
        }
        prev = t;
        t += step;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stroke(tool: BrushTool, points: Vec<[f32; 2]>) -> BrushStroke {
        BrushStroke { tool, radius: 10.0, strength: 1.0, points }
    }

    #[test]
    fn raise_and_lower_act_only_under_the_brush() {
        let mut f = Heightfield::flat(65, 0.5);
        apply_strokes(&mut f, &[stroke(BrushTool::Raise, vec![[0.0, 0.0]])], 100.0);
        assert!(f.get(32, 32) > 0.5);
        assert_eq!(f.get(0, 0), 0.5, "far away nothing moves");
        let mut g = Heightfield::flat(65, 0.5);
        apply_strokes(&mut g, &[stroke(BrushTool::Lower, vec![[0.0, 0.0]])], 100.0);
        assert!(g.get(32, 32) < 0.5);
    }

    #[test]
    fn heights_stay_between_zero_and_one() {
        let mut f = Heightfield::flat(33, 0.99);
        apply_strokes(&mut f, &[stroke(BrushTool::Raise, vec![[0.0, 0.0]; 200])], 100.0);
        assert_eq!(f.range().1, 1.0);
    }

    #[test]
    fn smooth_evens_out_a_spike() {
        let mut f = Heightfield::flat(33, 0.2);
        f.set(16, 16, 1.0);
        apply_strokes(&mut f, &[stroke(BrushTool::Smooth, vec![[0.0, 0.0]; 10])], 100.0);
        assert!(f.get(16, 16) < 0.6, "{}", f.get(16, 16));
    }

    #[test]
    fn flatten_levels_to_where_the_stroke_began() {
        let mut f = Heightfield::from_fn(65, || false, |u, _| Ok((u + 1.0) * 0.5)).unwrap();
        let start = f.sample(0.0, 0.0);
        let mut points = vec![[0.0, 0.0]];
        points.extend((0..30).map(|i| [i as f32 * 0.5, 0.0]));
        apply_strokes(&mut f, &[BrushStroke { radius: 6.0, ..stroke(BrushTool::Flatten, points) }], 100.0);
        assert!((f.sample(0.2, 0.0) - start).abs() < (0.6 - start).abs(), "the slope got closer to the start level");
    }

    #[test]
    fn strokes_are_deterministic_and_ignore_points_outside() {
        let s = [stroke(BrushTool::Raise, vec![[3.0, -4.0], [500.0, 500.0]])];
        let mut a = Heightfield::flat(33, 0.3);
        let mut b = Heightfield::flat(33, 0.3);
        apply_strokes(&mut a, &s, 100.0);
        apply_strokes(&mut b, &s, 100.0);
        assert_eq!(a, b);
    }

    #[test]
    fn a_ray_from_above_hits_the_ground() {
        let f = Heightfield::flat(65, 0.5);
        let hit = raycast(&f, [10.0, 100.0, -5.0], [0.0, -1.0, 0.0], 100.0, 20.0).unwrap();
        assert!((hit[0] - 10.0).abs() < 1e-3 && (hit[1] - 10.0).abs() < 0.05 && (hit[2] + 5.0).abs() < 1e-3, "{hit:?}");
    }

    #[test]
    fn a_slanted_ray_finds_the_hill_and_the_sky_misses() {
        let f = Heightfield::from_fn(65, || false, |u, v| Ok(1.0 - (u * u + v * v).sqrt().min(1.0))).unwrap();
        let hit = raycast(&f, [-80.0, 10.0, 0.0], [1.0, 0.0, 0.0], 100.0, 20.0).unwrap();
        assert!(hit[0] < 0.0 && hit[0] > -50.0, "it meets the slope on the near side: {hit:?}");
        assert!(raycast(&f, [-80.0, 10.0, 0.0], [0.0, 1.0, 0.0], 100.0, 20.0).is_none());
    }
}
