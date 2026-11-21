//! The data the shaders read, packed from a scene.
//!
//! The layouts here must match the structs at the top of `trace.wgsl` and
//! `resolve.wgsl`. Every field is a `vec4` so the WGSL alignment rules and
//! the Rust layout agree without padding tricks; the `layouts_match_wgsl`
//! test checks the sizes.

use bytemuck::{Pod, Zeroable};
use glam::{Mat3, Vec3};
use psito3d_scene::{MAX_OBJECTS, Scene};

use crate::Quality;

/// The `Globals` uniform of the trace pass.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Globals {
    /// Eye position; `w` is the tangent of half the field of view.
    pub cam_pos: [f32; 4],
    /// View direction; `w` is the aspect ratio.
    pub cam_fwd: [f32; 4],
    /// Screen right.
    pub cam_right: [f32; 4],
    /// Screen up.
    pub cam_up: [f32; 4],
    /// Towards the sun; `w` is the haze.
    pub sun_dir: [f32; 4],
    /// Cloud cover and cloud height in world units.
    pub sky: [f32; 4],
    /// Fog density, mist, mist height in world units.
    pub fog: [f32; 4],
    /// Sea on (1) or off (0), level in world units, ripple.
    pub sea: [f32; 4],
    /// Deep water colour.
    pub sea_color: [f32; 4],
    /// Terrain extent, height and heightmap resolution.
    pub terrain: [f32; 4],
    /// Traced width and height, and the sub-pixel jitter of this sample.
    pub res: [f32; 4],
    /// Frame index (samples so far) and object count.
    pub info: [u32; 4],
    /// March steps, shadow steps and bounces.
    pub quality: [u32; 4],
}

/// One entry of the `objects` storage buffer.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct GpuObject {
    /// First row of the world-to-object rotation; `w` is the position `x`.
    pub rot0: [f32; 4],
    /// Second row; `w` is the position `y`.
    pub rot1: [f32; 4],
    /// Third row; `w` is the position `z`.
    pub rot2: [f32; 4],
    /// The full size on each axis.
    pub size: [f32; 4],
    /// Colour, and metallic in `w`.
    pub color: [f32; 4],
    /// Roughness, transmission, index of refraction, blend radius.
    pub mat: [f32; 4],
    /// Shape, combine, visible.
    pub kind: [u32; 4],
}

/// The `Resolve` uniform of the resolve pass.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct ResolveParams {
    /// The exposure multiplier.
    pub params: [f32; 4],
    /// Accumulation width and height, output width and height.
    pub sizes: [f32; 4],
    /// The filter (`look` in the shader, since `filter` is reserved in WGSL).
    pub filter: [u32; 4],
}

fn v4(v: Vec3, w: f32) -> [f32; 4] {
    [v.x, v.y, v.z, w]
}

/// Packs the scene. `size` is the traced resolution and `heightmap_size`
/// the side of the uploaded heightfield. The frame index and the jitter are
/// left at zero; the renderer fills them per sample.
pub fn pack_globals(scene: &Scene, size: (u32, u32), heightmap_size: u32, quality: Quality) -> Globals {
    let cam = &scene.camera;
    let (fwd, right, up) = cam.basis();
    let aspect = size.0 as f32 / size.1.max(1) as f32;
    let height = scene.terrain.height.max(0.01);
    let objects = scene.objects.len().min(MAX_OBJECTS) as u32;
    Globals {
        cam_pos: v4(cam.eye(), (cam.fov.to_radians() * 0.5).tan()),
        cam_fwd: v4(fwd, aspect),
        cam_right: v4(right, 0.0),
        cam_up: v4(up, 0.0),
        sun_dir: v4(scene.sky.sun.direction(), scene.sky.haze),
        sky: [scene.sky.cloud_cover, scene.sky.cloud_height * height, 0.0, 0.0],
        fog: [scene.fog.density, scene.fog.mist, scene.fog.mist_height * height, 0.0],
        sea: [f32::from(u8::from(scene.sea.enabled)), scene.sea.level * height, scene.sea.ripple, 0.0],
        sea_color: [scene.sea.color[0], scene.sea.color[1], scene.sea.color[2], 0.0],
        terrain: [scene.terrain.extent, height, heightmap_size as f32, 0.0],
        res: [size.0 as f32, size.1 as f32, 0.0, 0.0],
        info: [0, objects, 0, 0],
        quality: [quality.march_steps(), quality.shadow_steps(), quality.bounces(), 0],
    }
}

/// Packs the objects, at most [`MAX_OBJECTS`].
pub fn pack_objects(scene: &Scene) -> Vec<GpuObject> {
    scene
        .objects
        .iter()
        .take(MAX_OBJECTS)
        .map(|o| {
            // The inverse of a rotation is its transpose: the rows of the
            // world-to-object rotation are the columns of the rotation.
            let r = Mat3::from_quat(o.transform.quat());
            let p = o.transform.position;
            let m = o.material.sanitized();
            GpuObject {
                rot0: v4(r.x_axis, p.x),
                rot1: v4(r.y_axis, p.y),
                rot2: v4(r.z_axis, p.z),
                size: v4(o.transform.scale.max(Vec3::splat(0.01)), 0.0),
                color: [m.color[0], m.color[1], m.color[2], m.metallic],
                mat: [m.roughness, m.transmission, m.ior, o.blend.max(0.0)],
                kind: [o.shape.gpu_index(), o.combine.gpu_index(), u32::from(o.visible), 0],
            }
        })
        .collect()
}

/// The sub-pixel offset of sample `n`, from the Halton (2, 3) sequence, so
/// the samples spread evenly over the pixel and smooth the edges.
pub fn jitter(n: u32) -> (f32, f32) {
    fn halton(mut i: u32, base: u32) -> f32 {
        let (mut f, mut r) = (1.0, 0.0);
        while i > 0 {
            f /= base as f32;
            r += f * (i % base) as f32;
            i /= base;
        }
        r
    }
    (halton(n + 1, 2), halton(n + 1, 3))
}

#[cfg(test)]
mod tests {
    use super::*;
    use psito3d_scene::{Material, Shape};

    #[test]
    fn layouts_match_wgsl() {
        // 13 vec4 in Globals, 7 in Object, 3 in Resolve.
        assert_eq!(std::mem::size_of::<Globals>(), 13 * 16);
        assert_eq!(std::mem::size_of::<GpuObject>(), 7 * 16);
        assert_eq!(std::mem::size_of::<ResolveParams>(), 3 * 16);
    }

    #[test]
    fn globals_follow_the_scene() {
        let mut scene = Scene::starter();
        scene.sea.enabled = false;
        let g = pack_globals(&scene, (800, 400), 256, Quality::Preview);
        assert_eq!(g.cam_fwd[3], 2.0);
        assert_eq!(g.sea[0], 0.0);
        assert_eq!(g.terrain[2], 256.0);
        assert_eq!(g.info[1], 1);
        assert_eq!(g.quality[0], Quality::Preview.march_steps());
        let sun = Vec3::from_slice(&g.sun_dir[..3]);
        assert!((sun.length() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn objects_carry_their_rotation_and_place() {
        let mut scene = Scene::default();
        let id = scene.add_object(Shape::Torus, Material::glass()).unwrap();
        let o = scene.object_mut(id).unwrap();
        o.transform.position = Vec3::new(1.0, 2.0, 3.0);
        o.transform.rotation = Vec3::new(0.0, 90.0, 0.0);
        o.visible = false;
        let g = pack_objects(&scene)[0];
        assert_eq!([g.rot0[3], g.rot1[3], g.rot2[3]], [1.0, 2.0, 3.0]);
        assert_eq!(g.kind, [Shape::Torus.gpu_index(), 0, 0, 0]);
        assert_eq!(g.mat[1], 1.0);
        // World +x is object -z after a 90° yaw: the first row maps x to local x.
        let local_of_x = Vec3::new(g.rot0[0], g.rot1[0], g.rot2[0]);
        assert!((local_of_x - Vec3::new(0.0, 0.0, 1.0)).length() < 1e-5, "{local_of_x}");
    }

    #[test]
    fn jitter_is_inside_the_pixel_and_varies() {
        let mut seen = std::collections::BTreeSet::new();
        for n in 0..64 {
            let (x, y) = jitter(n);
            assert!((0.0..1.0).contains(&x) && (0.0..1.0).contains(&y));
            seen.insert(((x * 1000.0) as u32, (y * 1000.0) as u32));
        }
        assert_eq!(seen.len(), 64);
    }
}
