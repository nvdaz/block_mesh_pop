use bevy_math::UVec3;

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
}

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
}
