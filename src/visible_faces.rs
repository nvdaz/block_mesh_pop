use seq_macro::seq;

use crate::{
    geometry::{quad::UnorientedUnitQuad, shape::ChunkShape},
    MeshVoxel, PopBuffer, UnorientedRegularQuad, VisitedBuffer, VoxelVisibility,
};

pub fn visible_faces_quads<
    const X: u32,
    const Y: u32,
    const Z: u32,
    const M: usize,
    V: MeshVoxel,
>(
    voxels: &[V],
    visited: &mut VisitedBuffer,
    output: &mut PopBuffer<M, UnorientedUnitQuad>,
) {
    assert_eq!(voxels.len(), (X * Y * Z) as usize);
    assert_eq!(voxels.len(), visited.visited.len());
    assert!(M <= u8::BITS as usize);
    assert!(M <= u8::BITS as usize);
    assert!(M <= X.ilog2() as usize);
    assert!(M <= Y.ilog2() as usize);
    assert!(M <= Z.ilog2() as usize);

    seq!(F in 0..6 {
        visited.reset();
        let face_strides = ChunkShape::<X,Y,Z>::FACE_STRIDES[F];

        for position in ChunkShape::<X, Y, Z>::inner_iter::<F>() {
            let index = ChunkShape::<X, Y, Z>::linearize(position);

            // SAFETY: `position` is within the interior of `ChunkShape`, and the `voxels` slice
            // must be the same size as `ChunkShape`, so index cannot be out-of-bounds.
            let voxel = unsafe { voxels.get_unchecked(index as usize) };

            if voxel.get_visibility() != VoxelVisibility::Empty {
                let neighbor_index = index.wrapping_add(face_strides.n);

                // SAFETY: `index` refers to `position` which is in the interior (not on the edge)
                // of `ChunkShape`. Moving `index` one unit in any direction (by adding the normal stride)
                // will keep `neighbor_index` within the shape of `ChunkShape`, which is
                // the same size as `voxels`.
                let neighbor_voxel = unsafe { voxels.get_unchecked(neighbor_index as usize) };

                let face_needs_mesh = match neighbor_voxel.get_visibility() {
                    VoxelVisibility::Empty => true,
                    VoxelVisibility::Translucent => {
                        voxel.get_visibility() == VoxelVisibility::Opaque
                    }
                    VoxelVisibility::Opaque => false,
                };

                if face_needs_mesh {
                    let quad = UnorientedUnitQuad { minimum: position };

                    // SAFETY: This is safe for the  same rason the `voxel` access is safe;
                    // `voxels` and `visited` are checked to have the same length.
                    let lod = unsafe { find_max_lod::<X, Y, Z, M>(&mut visited.visited, quad) };

                    output.add_quad(F, quad, lod)
                }
            }
        }
    });
}

#[inline]
fn into_lod(quad: UnorientedUnitQuad, lod: usize) -> UnorientedRegularQuad {
    UnorientedRegularQuad {
        minimum: (((quad.minimum - 1) >> (lod as u32)) << (lod as u32)) + 1,
        size: 1 << lod,
    }
}

/// Finds the maximum level of detail at which a quad will not overlap a pre-existing quad.
///
/// # Safety
///
/// The minimum position of `quad` must be able to fit within the visited buffer.
#[inline]
unsafe fn find_max_lod<const X: u32, const Y: u32, const Z: u32, const M: usize>(
    visited: &mut [u8],
    quad: UnorientedUnitQuad,
) -> usize {
    let mut max_lod: usize = 0;

    for lod in (0..M).rev() {
        let quad_lod = into_lod(quad, lod);
        let index = ChunkShape::<X, Y, Z>::linearize(quad_lod.minimum);

        // Unit quads will never partially overlap eachother at any level of detail;
        // therefore, it suffices to only check the minimum index of the quad.
        //
        // SAFETY: The caller gaurantees that the minimum of the original quad will fit within the
        // visited buffer. The minimum of the LOD-adjusted quad will always be at most
        // that of the original quad. Therefore, the minimum index of the LOD-adjusted quad
        // will be within the bounds of `visited`.
        let visited_index = unsafe { visited.get_unchecked_mut(index as usize) };
        if *visited_index & (1 << lod) == 0 {
            *visited_index |= 1 << lod;

            if max_lod < lod {
                max_lod = lod;
            }
        }
    }

    max_lod
}
