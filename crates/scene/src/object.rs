//! The objects placed in the scene: primitives with a material.

use glam::{EulerRot, Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

/// A stable id, so the editor can keep a selection while the list changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ObjectId(pub u64);

/// The primitive shapes. Each one fits a unit cube centred on the origin
/// before the [`Transform`] scales it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Shape {
    /// A ball.
    Sphere,
    /// A box with slightly rounded edges.
    Cube,
    /// A cylinder standing on `y`.
    Cylinder,
    /// A cone with the tip up.
    Cone,
    /// A ring lying flat.
    Torus,
}

impl Shape {
    /// Every shape, in the order of the editor's menu.
    pub const ALL: [Shape; 5] = [Shape::Sphere, Shape::Cube, Shape::Cylinder, Shape::Cone, Shape::Torus];

    /// The index the shader uses.
    pub fn gpu_index(self) -> u32 {
        self as u32
    }
}

/// How an object joins the ones before it in the list.
///
/// With a [`SceneObject::blend`] above zero the joint is smooth: two balls
/// melt into each other like drops of mercury.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Combine {
    /// Adds the object.
    #[default]
    Union,
    /// Carves the object out of what came before.
    Subtract,
    /// Keeps only where the object and what came before overlap.
    Intersect,
}

impl Combine {
    /// Every operation, in the order of the editor's menu.
    pub const ALL: [Combine; 3] = [Combine::Union, Combine::Subtract, Combine::Intersect];

    /// The index the shader uses.
    pub fn gpu_index(self) -> u32 {
        self as u32
    }
}

/// Position, rotation and size.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Transform {
    /// The centre, in world units.
    pub position: Vec3,
    /// The rotation, in degrees, applied as yaw (`y`), pitch (`x`), roll (`z`).
    pub rotation: Vec3,
    /// The size along each axis, in world units.
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self { position: Vec3::ZERO, rotation: Vec3::ZERO, scale: Vec3::splat(4.0) }
    }
}

impl Transform {
    /// The rotation as a quaternion.
    pub fn quat(&self) -> Quat {
        let r = self.rotation * (std::f32::consts::PI / 180.0);
        Quat::from_euler(EulerRot::YXZ, r.y, r.x, r.z)
    }

    /// Object space to world space, scale included.
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale.max(Vec3::splat(1e-3)), self.quat(), self.position)
    }
}

/// What a surface is made of. Colours are in linear RGB.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Material {
    /// The base colour.
    pub color: [f32; 3],
    /// 0 for paint and stone, 1 for metal: a metal reflects in its own colour.
    pub metallic: f32,
    /// 0 is a perfect mirror, 1 is chalk.
    pub roughness: f32,
    /// 0 is opaque, 1 is clear glass.
    pub transmission: f32,
    /// How much glass bends the light; 1.5 is window glass.
    pub ior: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self::clay([0.8, 0.8, 0.8])
    }
}

impl Material {
    /// The mirror-like chrome of every 90s render.
    pub fn chrome() -> Self {
        Self { color: [0.95, 0.95, 0.97], metallic: 1.0, roughness: 0.02, transmission: 0.0, ior: 1.5 }
    }

    /// Polished gold.
    pub fn gold() -> Self {
        Self { color: [1.0, 0.71, 0.29], metallic: 1.0, roughness: 0.12, transmission: 0.0, ior: 1.5 }
    }

    /// Clear glass.
    pub fn glass() -> Self {
        Self { color: [0.97, 0.99, 1.0], metallic: 0.0, roughness: 0.0, transmission: 1.0, ior: 1.5 }
    }

    /// A matte surface of `color`.
    pub fn clay(color: [f32; 3]) -> Self {
        Self { color, metallic: 0.0, roughness: 0.85, transmission: 0.0, ior: 1.5 }
    }

    /// The material with every field in its valid range.
    pub fn sanitized(self) -> Self {
        let c = |v: f32| if v.is_finite() { v.clamp(0.0, 1.0) } else { 0.5 };
        Self {
            color: self.color.map(c),
            metallic: c(self.metallic),
            roughness: c(self.roughness),
            transmission: c(self.transmission),
            ior: if self.ior.is_finite() { self.ior.clamp(1.0, 3.0) } else { 1.5 },
        }
    }
}

/// An object in the scene.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneObject {
    /// The stable id.
    pub id: ObjectId,
    /// The name in the object list.
    pub name: String,
    /// The primitive.
    pub shape: Shape,
    /// Where it is and how big.
    #[serde(default)]
    pub transform: Transform,
    /// What it is made of.
    #[serde(default)]
    pub material: Material,
    /// How it joins the objects before it.
    #[serde(default)]
    pub combine: Combine,
    /// The radius of the smooth joint, in world units; 0 is a sharp joint.
    #[serde(default)]
    pub blend: f32,
    /// Hidden objects are kept but not drawn.
    #[serde(default = "visible")]
    pub visible: bool,
}

fn visible() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_clamps_everything() {
        let m = Material { color: [2.0, f32::NAN, -1.0], metallic: 3.0, roughness: -1.0, transmission: 0.5, ior: 9.0 }
            .sanitized();
        assert_eq!(m.color, [1.0, 0.5, 0.0]);
        assert_eq!((m.metallic, m.roughness, m.ior), (1.0, 0.0, 3.0));
    }

    #[test]
    fn matrix_places_the_centre() {
        let t = Transform { position: Vec3::new(1.0, 2.0, 3.0), ..Default::default() };
        assert_eq!(t.matrix().transform_point3(Vec3::ZERO), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn yaw_turns_around_y() {
        let t = Transform { rotation: Vec3::new(0.0, 90.0, 0.0), scale: Vec3::ONE, ..Default::default() };
        let p = t.matrix().transform_point3(Vec3::X);
        assert!((p - Vec3::new(0.0, 0.0, -1.0)).length() < 1e-5, "{p}");
    }
}
