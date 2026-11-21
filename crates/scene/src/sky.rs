//! The sky, the fog and the sea.

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Where the sun is, as the person drags it around the sky.
///
/// The sun is stored as a direction and not as a clock time, the way the
/// person moves it in the editor; [`Sun::from_time_of_day`] turns a clock
/// time into a direction for the time-of-day slider.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Sun {
    /// The compass heading, in degrees: 0 looks along `-z`, 90 along `+x`.
    pub azimuth: f32,
    /// The angle above the horizon, in degrees, from -90 to 90.
    pub elevation: f32,
}

impl Default for Sun {
    fn default() -> Self {
        Self { azimuth: 300.0, elevation: 9.0 }
    }
}

impl Sun {
    /// The unit vector pointing *towards* the sun.
    pub fn direction(&self) -> Vec3 {
        let az = self.azimuth.to_radians();
        let el = self.elevation.clamp(-90.0, 90.0).to_radians();
        Vec3::new(el.cos() * az.sin(), el.sin(), -el.cos() * az.cos())
    }

    /// The sun for a direction (not necessarily normalised).
    pub fn from_direction(dir: Vec3) -> Self {
        let d = dir.normalize_or(Vec3::Y);
        let elevation = d.y.clamp(-1.0, 1.0).asin().to_degrees();
        let azimuth = d.x.atan2(-d.z).to_degrees().rem_euclid(360.0);
        Self { azimuth, elevation }
    }

    /// The sun at `hours` (0 to 24) of a day whose noon sun is at
    /// `noon_azimuth` and `noon_elevation`. It rises at 6 and sets at 18,
    /// a simple model that is right for an equinox and is all a toy needs.
    pub fn from_time_of_day(hours: f32, noon_azimuth: f32, noon_elevation: f32) -> Self {
        // The hour angle: 0 at noon, ±90° at 6 and 18.
        let h = ((hours.rem_euclid(24.0) - 12.0) / 12.0) * std::f32::consts::PI;
        let elevation = (noon_elevation.to_radians().sin() * h.cos()).asin().to_degrees();
        let azimuth = (noon_azimuth + h.to_degrees()).rem_euclid(360.0);
        Self { azimuth, elevation }
    }

    /// Whether the sun is below the horizon.
    pub fn is_night(&self) -> bool {
        self.elevation < -4.0
    }
}

/// The sky above the scene.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Sky {
    /// The sun.
    pub sun: Sun,
    /// How much of the sky the cloud layer covers, from 0 to 1.
    pub cloud_cover: f32,
    /// The height of the cloud layer, in multiples of the terrain height.
    pub cloud_height: f32,
    /// How hazy the air is, from 0 (crisp) to 1 (milky). It warms up the
    /// sunset and pales the horizon.
    pub haze: f32,
    /// Brightness of the final image, in stops.
    pub exposure: f32,
}

impl Default for Sky {
    fn default() -> Self {
        Self { sun: Sun::default(), cloud_cover: 0.45, cloud_height: 4.0, haze: 0.12, exposure: 0.0 }
    }
}

/// Fog by distance and a mist that settles in the valleys.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Fog {
    /// How thick the fog by distance is, from 0 to 1.
    pub density: f32,
    /// How thick the low mist is, from 0 to 1.
    pub mist: f32,
    /// The height the mist reaches, in multiples of the terrain height.
    pub mist_height: f32,
}

impl Default for Fog {
    fn default() -> Self {
        Self { density: 0.15, mist: 0.15, mist_height: 0.25 }
    }
}

/// The endless sea, a mirror up to the horizon.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Sea {
    /// Without the sea there is an endless ground plane at height zero.
    pub enabled: bool,
    /// The water level, from 0 to 1 of the terrain height.
    pub level: f32,
    /// The colour of deep water, in linear RGB.
    pub color: [f32; 3],
    /// How rough the surface is, from a mirror (0) to choppy (1).
    pub ripple: f32,
}

impl Default for Sea {
    fn default() -> Self {
        Self { enabled: true, level: 0.18, color: [0.02, 0.07, 0.10], ripple: 0.25 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_round_trips() {
        for (az, el) in [(0.0, 10.0), (90.0, 45.0), (200.0, 18.0), (359.0, -20.0)] {
            let sun = Sun { azimuth: az, elevation: el };
            let back = Sun::from_direction(sun.direction());
            assert!((back.azimuth - az).abs() < 1e-3, "{az} -> {}", back.azimuth);
            assert!((back.elevation - el).abs() < 1e-3);
        }
    }

    #[test]
    fn direction_is_unit_and_points_up_at_noon() {
        let d = Sun { azimuth: 0.0, elevation: 90.0 }.direction();
        assert!((d.length() - 1.0).abs() < 1e-5);
        assert!(d.y > 0.999);
    }

    #[test]
    fn time_of_day_rises_and_sets() {
        let noon = Sun::from_time_of_day(12.0, 180.0, 60.0);
        assert!((noon.elevation - 60.0).abs() < 1e-3);
        assert!((noon.azimuth - 180.0).abs() < 1e-3);
        assert!(Sun::from_time_of_day(6.0, 180.0, 60.0).elevation.abs() < 1e-3);
        assert!(Sun::from_time_of_day(0.0, 180.0, 60.0).is_night());
        let morning = Sun::from_time_of_day(9.0, 180.0, 60.0);
        let evening = Sun::from_time_of_day(15.0, 180.0, 60.0);
        assert!((morning.elevation - evening.elevation).abs() < 1e-3);
        assert!(morning.azimuth < 180.0 && evening.azimuth > 180.0);
    }
}

/// A finishing filter laid over the final image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Filter {
    /// The image as rendered.
    #[default]
    None,
    /// 256 colours with ordered dithering, like a 90s screen.
    Dither,
    /// The scanlines and the curved glow of a tube monitor.
    Crt,
}

impl Filter {
    /// Every filter, in the order of the editor's menu.
    pub const ALL: [Filter; 3] = [Filter::None, Filter::Dither, Filter::Crt];

    /// The index the shader uses.
    pub fn gpu_index(self) -> u32 {
        self as u32
    }
}
