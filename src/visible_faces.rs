use std::cmp;

use crate::{
    geometry::{face::OrientedBlockFace, quad::UnorientedUnitQuad, shape::ChunkShape},
    MeshVoxel, PopBuffer, QuadsBuffer, UnorientedRegularQuad, VoxelVisibility,
};

pub fn visible_faces_quads<
    const X: u32,
    const Y: u32,
    const Z: u32,
    const M: usize,
    V: MeshVoxel,
>(
    voxels: &[V],
    output: &mut PopBuffer<M, UnorientedUnitQuad>,
) {
    let strides = OrientedBlockFace::FACES.map(|face| {
        (
            face,
            ChunkShape::<X, Y, Z>::linearize(face.signed_normal().as_uvec3()),
        )
    });

    for position in ChunkShape::<X, Y, Z>::inner_iter() {
        let index = ChunkShape::<X, Y, Z>::linearize(position);
        let voxel = unsafe { voxels.get_unchecked(index as usize) };

        if voxel.get_visibility() != VoxelVisibility::Empty {
            for (face_indx, (face, face_stride)) in strides.iter().enumerate() {
                let neighbor_index = index.wrapping_add(*face_stride);
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
                    let lod = max_lod::<M>(quad, *face);

                    output.add_quad(face_indx, quad, lod)
                }
            }
        }
    }
}

#[inline]//TODO: remove pub
pub(crate) fn into_lod<const M: usize>(quad: UnorientedUnitQuad, lod: usize) -> UnorientedRegularQuad {
    UnorientedRegularQuad {
        minimum: (quad.minimum >> (lod as u32)) << (lod as u32),
        size: 1 << lod,
    }
}

#[inline]
fn max_lod<const M: usize>(quad: UnorientedUnitQuad, face: OrientedBlockFace) -> usize {
    let u_pos = quad.minimum.dot(face.u);
    let v_pos = quad.minimum.dot(face.v);

    (1 + cmp::min(u_pos.trailing_zeros(), v_pos.trailing_zeros()) as usize).min(M - 1)
}

pub fn extract_lod_buffer<const M: usize>(
    pop_buffer: &PopBuffer<M, UnorientedUnitQuad>,
    output_buffer: &mut QuadsBuffer<UnorientedRegularQuad>,
    lod: usize,
) {
    for (face_indx, group) in pop_buffer.groups.iter().enumerate() {
        let to_len = group.buckets[lod];
        output_buffer.groups[face_indx].extend(
            group.quads[0..to_len]
                .iter()
                .map(|&quad| into_lod::<M>(quad, lod)),
        );
    }
}
