use bevy_math::{IVec3, UVec3, Vec3};

use super::{axis::AxisPermutation, quad::UnorientedQuad};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrientedBlockFace {
    pub n_sign: i32,
    pub parity: i32,
    pub n: UVec3,
    pub u: UVec3,
    pub v: UVec3,
}

impl OrientedBlockFace {
    pub const FACES: [Self; 6] = [
        Self::new(-1, AxisPermutation::Xzy),
        Self::new(-1, AxisPermutation::Yzx),
        Self::new(-1, AxisPermutation::Zxy),
        Self::new(1, AxisPermutation::Xzy),
        Self::new(1, AxisPermutation::Yzx),
        Self::new(1, AxisPermutation::Zxy),
    ];

    const fn new(n_sign: i32, permutation: AxisPermutation) -> Self {
        let [n_axis, u_axis, v_axis] = permutation.axes();
        Self {
            n_sign,
            parity: permutation.parity(),
            n: n_axis.get_unit_vector(),
            u: u_axis.get_unit_vector(),
            v: v_axis.get_unit_vector(),
        }
    }

    #[inline]
    pub(crate) fn signed_normal(&self) -> IVec3 {
        self.n.as_ivec3() * self.n_sign
    }

    #[inline]
    pub fn quad_mesh_indices(&self, start: u32) -> [u32; 6] {
        let counter_clockwise = self.n_sign * self.parity > 0;

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

        let minu_minv = if self.n_sign > 0 {
            quad.minimum + n_vec
        } else {
            quad.minimum
        };
        let maxu_minv = minu_minv + w_vec;
        let minu_maxv = minu_minv + h_vec;
        let maxu_maxv = minu_minv + w_vec + h_vec;

        // println!("C- {:?} {:?}", minu_minv, maxu_maxv);

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
        [self.signed_normal().as_vec3(); 4]
    }
}
