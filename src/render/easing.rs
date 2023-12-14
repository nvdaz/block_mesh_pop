use bevy::render::render_resource::ShaderDefVal;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LodEasing {
    Linear,
    Quadratic,
    Cubic,
    Sine,
}

impl LodEasing {
    pub(crate) fn calculate(&self, distance: f32, period: u32, max_lod: u32) -> f32 {
        let period = period as f32;
        let max_lod = max_lod as f32;
        let distance = distance.clamp(0.0, period);
        match *self {
            Self::Linear => max_lod / period * distance,
            Self::Quadratic => max_lod * (distance / period).powi(2),
            Self::Cubic => max_lod * (distance / period).powi(3),
            Self::Sine => {
                max_lod - max_lod * (std::f32::consts::PI * distance / (2.0 * period)).cos()
            }
        }
    }
}

impl From<LodEasing> for ShaderDefVal {
    fn from(value: LodEasing) -> Self {
        let name = match value {
            LodEasing::Linear => "EASING_LINEAR",
            LodEasing::Quadratic => "EASING_QUADRATIC",
            LodEasing::Cubic => "EASING_CUBIC",
            LodEasing::Sine => "EASING_SINE",
        };

        Self::Bool(name.into(), true)
    }
}
