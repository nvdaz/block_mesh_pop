use bevy_math::{IVec3, UVec3, Vec3};

use super::{axis::AxisPermutation, quad::UnorientedQuad};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrientedBlockFace {
    pub(crate) is_front: bool,
    pub(crate) is_even: bool,
    pub(crate) n_sign: i32,
    pub signed_n: IVec3,
    pub n: UVec3,
    pub u: UVec3,
    pub v: UVec3,
}

impl OrientedBlockFace {
    pub const FACES: [Self; 6] = [
        Self::new(false, AxisPermutation::Xzy),
        Self::new(false, AxisPermutation::Yzx),
        Self::new(false, AxisPermutation::Zxy),
        Self::new(true, AxisPermutation::Xzy),
        Self::new(true, AxisPermutation::Yzx),
        Self::new(true, AxisPermutation::Zxy),
    ];

    #[inline]
    const fn new(is_front: bool, permutation: AxisPermutation) -> Self {
        let [n_axis, u_axis, v_axis] = permutation.axes();
        Self {
            is_front: is_front,
            is_even: permutation.is_even(),
            n_sign: if is_front { 1 } else { -1 },
            signed_n: n_axis.get_signed_vector(is_front),
            n: n_axis.get_unit_vector(),
            u: u_axis.get_unit_vector(),
            v: v_axis.get_unit_vector(),
        }
    }

    #[inline]
    pub const fn quad_mesh_indices(&self, start: u32) -> [u32; 6] {
        let counter_clockwise = self.is_front == self.is_even;

        if counter_clockwise {
            [start, start + 1, start + 2, start + 1, start + 3, start + 2]
        } else {
            [start, start + 2, start + 1, start + 1, start + 2, start + 3]
        }
    }

    #[inline]
    pub fn quad_corners(&self, quad: impl Into<UnorientedQuad>, lod: usize) -> [UVec3; 4] {
        let quad = quad.into();
        let n_vec = self.n * (1 << lod as u32);
        let w_vec = self.u * quad.width;
        let h_vec = self.v * quad.height;

        let minu_minv = if self.is_front {
            quad.minimum + n_vec
        } else {
            quad.minimum
        };
        let maxu_minv = minu_minv + w_vec;
        let minu_maxv = minu_minv + h_vec;
        let maxu_maxv = minu_minv + w_vec + h_vec;

        [minu_minv, maxu_minv, minu_maxv, maxu_maxv]
    }

    #[inline]
    pub fn quad_mesh_positions(
        &self,
        quad: impl Into<UnorientedQuad>,
        lod: usize,
        voxel_size: f32,
    ) -> [Vec3; 4] {
        self.quad_corners(quad, lod)
            .map(|corner| (voxel_size * corner.as_vec3()))
    }

    #[inline]
    pub fn quad_mesh_normals(&self) -> [Vec3; 4] {
        [self.signed_n.as_vec3(); 4]
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FaceStrides {
    pub(crate) n: u32,
    pub(crate) u: u32,
    pub(crate) v: u32,
}
