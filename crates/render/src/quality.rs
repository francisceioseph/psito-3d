//! How much work each sample does.

/// The two speeds of the renderer.
///
/// While the person drags something the renderer draws [`Quality::Preview`]
/// frames: half the resolution, fewer steps, short reflections, and a fresh
/// frame every time. When they let go it switches to [`Quality::Final`] and
/// keeps adding samples until the image is clean.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Quality {
    /// Fast and rough, for interaction.
    Preview,
    /// Slow and clean, accumulated over many frames.
    #[default]
    Final,
}

impl Quality {
    /// The most ray marching steps per ray.
    pub fn march_steps(self) -> u32 {
        match self {
            Self::Preview => 140,
            Self::Final => 320,
        }
    }

    /// The most steps of a shadow ray.
    pub fn shadow_steps(self) -> u32 {
        match self {
            Self::Preview => 28,
            Self::Final => 80,
        }
    }

    /// How many mirror, glass and water bounces a path follows.
    pub fn bounces(self) -> u32 {
        match self {
            Self::Preview => 2,
            Self::Final => 5,
        }
    }

    /// The fraction of the output resolution traced.
    pub fn scale(self) -> f32 {
        match self {
            Self::Preview => 0.5,
            Self::Final => 1.0,
        }
    }
}
