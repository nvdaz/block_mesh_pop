use bevy_math::UVec3;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct UnorientedQuad {
    pub minimum: UVec3,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct UnorientedRegularQuad {
    pub minimum: UVec3,
    pub size: u32,
}

impl From<UnorientedRegularQuad> for UnorientedQuad {
    fn from(value: UnorientedRegularQuad) -> Self {
        Self {
            minimum: value.minimum,
            width: value.size,
            height: value.size,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct UnorientedUnitQuad {
    pub minimum: UVec3,
}

impl From<UnorientedUnitQuad> for UnorientedRegularQuad {
    fn from(value: UnorientedUnitQuad) -> Self {
        Self {
            minimum: value.minimum,
            size: 1,
        }
    }
}

impl From<UnorientedUnitQuad> for UnorientedQuad {
    fn from(value: UnorientedUnitQuad) -> Self {
        Self {
            minimum: value.minimum,
            width: 1,
            height: 1,
        }
    }
}
