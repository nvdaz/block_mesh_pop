use bevy_math::{IVec3, UVec3};

pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    #[inline]
    pub const fn get_unit_vector(&self) -> UVec3 {
        match self {
            Axis::X => UVec3::X,
            Axis::Y => UVec3::Y,
            Axis::Z => UVec3::Z,
        }
    }

    #[inline]
    pub const fn get_signed_vector(&self, is_front: bool) -> IVec3 {
        match (self, is_front) {
            (Axis::X, true) => IVec3::X,
            (Axis::X, false) => IVec3::NEG_X,
            (Axis::Y, true) => IVec3::Y,
            (Axis::Y, false) => IVec3::NEG_Y,
            (Axis::Z, true) => IVec3::Z,
            (Axis::Z, false) => IVec3::NEG_Z,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxisPermutation {
    Xyz,
    Zxy,
    Yzx,
    Zyx,
    Xzy,
    Yxz,
}

impl AxisPermutation {
    #[inline]
    pub const fn parity(&self) -> i32 {
        match self {
            AxisPermutation::Xyz => 1,
            AxisPermutation::Zxy => 1,
            AxisPermutation::Yzx => 1,
            AxisPermutation::Zyx => -1,
            AxisPermutation::Xzy => -1,
            AxisPermutation::Yxz => -1,
        }
    }

    #[inline]
    pub const fn is_even(&self) -> bool {
        match self {
            AxisPermutation::Xyz => true,
            AxisPermutation::Zxy => true,
            AxisPermutation::Yzx => true,
            AxisPermutation::Zyx => false,
            AxisPermutation::Xzy => false,
            AxisPermutation::Yxz => false,
        }
    }

    #[inline]
    pub const fn axes(&self) -> [Axis; 3] {
        match self {
            AxisPermutation::Xyz => [Axis::X, Axis::Y, Axis::Z],
            AxisPermutation::Zxy => [Axis::Z, Axis::X, Axis::Y],
            AxisPermutation::Yzx => [Axis::Y, Axis::Z, Axis::X],
            AxisPermutation::Zyx => [Axis::Z, Axis::Y, Axis::X],
            AxisPermutation::Xzy => [Axis::X, Axis::Z, Axis::Y],
            AxisPermutation::Yxz => [Axis::Y, Axis::X, Axis::Z],
        }
    }

    #[inline]
    pub const fn n_axis(&self) -> Axis {
        match self {
            AxisPermutation::Xyz | AxisPermutation::Xzy => Axis::X,
            AxisPermutation::Yxz | AxisPermutation::Yzx => Axis::Y,
            AxisPermutation::Zxy | AxisPermutation::Zyx => Axis::Z,
        }
    }

    #[inline]
    pub const fn u_axis(&self) -> Axis {
        match self {
            AxisPermutation::Yxz | AxisPermutation::Zxy => Axis::X,
            AxisPermutation::Xyz | AxisPermutation::Zyx => Axis::Y,
            AxisPermutation::Xzy | AxisPermutation::Yzx => Axis::Z,
        }
    }

    #[inline]
    pub const fn v_axis(&self) -> Axis {
        match self {
            AxisPermutation::Yzx | AxisPermutation::Zyx => Axis::X,
            AxisPermutation::Xzy | AxisPermutation::Zxy => Axis::Y,
            AxisPermutation::Xyz | AxisPermutation::Yxz => Axis::Z,
        }
    }
}
