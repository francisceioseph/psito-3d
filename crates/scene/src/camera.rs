//! The camera, which orbits a target point.

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// An orbit camera: it always looks at `target`, from `distance` away, at the
/// angles `yaw` and `pitch`. This is the easiest camera to play with: drag to
/// turn around the island, scroll to come closer.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Camera {
    /// The point looked at.
    pub target: Vec3,
    /// How far the eye is from the target.
    pub distance: f32,
    /// The turn around `y`, in degrees.
    pub yaw: f32,
    /// The angle above the target, in degrees.
    pub pitch: f32,
    /// The vertical field of view, in degrees.
    pub fov: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self { target: Vec3::new(0.0, 6.0, 0.0), distance: 95.0, yaw: 25.0, pitch: 12.0, fov: 45.0 }
    }
}

impl Camera {
    /// The limits of the pitch, short of straight up and down.
    pub const PITCH_RANGE: (f32, f32) = (-80.0, 89.0);
    /// The limits of the distance.
    pub const DISTANCE_RANGE: (f32, f32) = (2.0, 2000.0);

    /// Where the eye is.
    pub fn eye(&self) -> Vec3 {
        let yaw = self.yaw.to_radians();
        let pitch = self.pitch.to_radians();
        let offset = Vec3::new(pitch.cos() * yaw.sin(), pitch.sin(), pitch.cos() * yaw.cos());
        self.target + offset * self.distance
    }

    /// The unit vectors `(forward, right, up)` of the view.
    pub fn basis(&self) -> (Vec3, Vec3, Vec3) {
        let forward = (self.target - self.eye()).normalize_or(Vec3::NEG_Z);
        let right = forward.cross(Vec3::Y).normalize_or(Vec3::X);
        let up = right.cross(forward);
        (forward, right, up)
    }

    /// The ray through a point of the image, as the renderer traces it:
    /// `ndc` goes from -1 to 1 left to right and bottom to top, and `aspect`
    /// is the image width over its height. Returns the eye and the unit
    /// direction.
    pub fn ray(&self, ndc: (f32, f32), aspect: f32) -> (Vec3, Vec3) {
        let (forward, right, up) = self.basis();
        let t = (self.fov.to_radians() * 0.5).tan();
        let dir = forward + right * ndc.0 * t * aspect + up * ndc.1 * t;
        (self.eye(), dir.normalize_or(forward))
    }

    /// Turns around the target by the given degrees.
    pub fn orbit(&mut self, d_yaw: f32, d_pitch: f32) {
        self.yaw = (self.yaw + d_yaw).rem_euclid(360.0);
        self.pitch = (self.pitch + d_pitch).clamp(Self::PITCH_RANGE.0, Self::PITCH_RANGE.1);
    }

    /// Moves closer (`factor` < 1) or farther (`factor` > 1).
    pub fn zoom(&mut self, factor: f32) {
        self.distance = (self.distance * factor).clamp(Self::DISTANCE_RANGE.0, Self::DISTANCE_RANGE.1);
    }

    /// Slides the target sideways and up, in screen directions, by a fraction
    /// of the distance, so panning feels the same near and far.
    pub fn pan(&mut self, dx: f32, dy: f32) {
        let (_, right, up) = self.basis();
        self.target += (right * dx + up * dy) * self.distance;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eye_is_at_the_distance() {
        let c = Camera::default();
        assert!(((c.eye() - c.target).length() - c.distance).abs() < 1e-3);
    }

    #[test]
    fn basis_is_orthonormal() {
        let (f, r, u) = Camera::default().basis();
        for v in [f, r, u] {
            assert!((v.length() - 1.0).abs() < 1e-5);
        }
        assert!(f.dot(r).abs() < 1e-5 && f.dot(u).abs() < 1e-5 && r.dot(u).abs() < 1e-5);
        assert!(u.y > 0.0, "up points up");
    }

    #[test]
    fn the_middle_ray_looks_at_the_target() {
        let c = Camera::default();
        let (eye, dir) = c.ray((0.0, 0.0), 1.5);
        assert!((eye + dir * c.distance - c.target).length() < 1e-3);
        let (_, top) = c.ray((0.0, 1.0), 1.5);
        assert!(top.y > dir.y, "the top of the image looks higher");
    }

    #[test]
    fn orbit_and_zoom_stay_in_range() {
        let mut c = Camera::default();
        c.orbit(400.0, 500.0);
        assert!((0.0..360.0).contains(&c.yaw));
        assert_eq!(c.pitch, Camera::PITCH_RANGE.1);
        c.zoom(1e-6);
        assert_eq!(c.distance, Camera::DISTANCE_RANGE.0);
    }
}
