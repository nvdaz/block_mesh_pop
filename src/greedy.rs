use bevy_math::UVec3;

use crate::{
    geometry::face::OrientedBlockFace, ChunkShape, MergeVoxel, MeshVoxel, PopBuffer,
    UnorientedQuad, VisitedBuffer, VoxelVisibility,
};

pub fn greedy_quads<const X: u32, const Y: u32, const Z: u32, const M: usize, V: MergeVoxel>(
    voxels: &[V],
    visited: &mut VisitedBuffer,
    pop_buffer: &mut PopBuffer<M, UnorientedQuad>,
) {
    assert_eq!(voxels.len(), (X * Y * Z) as usize);
    assert_eq!(voxels.len(), visited.visited.len());
    assert!(M <= u8::BITS as usize);
    assert!(M <= X.ilog2() as usize);
    assert!(M <= Y.ilog2() as usize);
    assert!(M <= Z.ilog2() as usize);

    for (face_index, face) in OrientedBlockFace::FACES.into_iter().enumerate() {
        visited.reset();
        let interior_shape = UVec3::new(X - 2, Y - 2, Z - 2);

        let n_max = face.n.dot(interior_shape) + 1;
        let u_max = face.u.dot(interior_shape) + 1;
        let v_max = face.v.dot(interior_shape) + 1;
        let n_stride = ChunkShape::<X, Y, Z>::linearize(face.signed_n.as_uvec3());
        let u_stride = ChunkShape::<X, Y, Z>::linearize(face.u);
        let v_stride = ChunkShape::<X, Y, Z>::linearize(face.v);

        for n in 1..n_max {
            for position in ChunkShape::<X, Y, Z>::slice_iter(face, n) {
                let index = ChunkShape::<X, Y, Z>::linearize(position);
                let voxel = unsafe { voxels.get_unchecked(index as usize) };

                let neighbor_index = index.wrapping_add(n_stride);
                let neighbor_voxel = unsafe { voxels.get_unchecked(neighbor_index as usize) };

                if face_needs_mesh(&visited.visited, index, voxel, neighbor_voxel) {
                    let max_width = u_max - face.u.dot(position);
                    let max_height = v_max - face.v.dot(position);

                    let merge_value = voxel.merge_value();
                    let merge_neighbor_value = neighbor_voxel.merge_value_facing_neighbour();

                    let width = get_max_width(
                        voxels,
                        &visited.visited,
                        &merge_value,
                        &merge_neighbor_value,
                        index,
                        n_stride,
                        u_stride,
                        max_width,
                    );

                    let height = get_max_height(
                        voxels,
                        &visited.visited,
                        &merge_value,
                        &merge_neighbor_value,
                        index + v_stride,
                        n_stride,
                        u_stride,
                        v_stride,
                        width,
                        max_height,
                    );

                    let quad = UnorientedQuad {
                        minimum: position,
                        width,
                        height,
                    };

                    let lod = find_max_lod::<X, Y, Z, M>(
                        &mut visited.visited,
                        quad,
                        face,
                        u_stride,
                        v_stride,
                    );

                    mark_visited(&mut visited.visited, quad, index, u_stride, v_stride, 0);

                    pop_buffer.add_quad(face_index, quad, lod)
                }
            }
        }
    }
}

#[inline]
fn face_needs_mesh<T>(visited: &[u8], index: u32, voxel: &T, neighbor: &T) -> bool
where
    T: MeshVoxel,
{
    voxel.get_visibility() != VoxelVisibility::Empty
        && visited[index as usize] & 1 == 0
        && match neighbor.get_visibility() {
            VoxelVisibility::Empty => true,
            VoxelVisibility::Translucent => voxel.get_visibility() == VoxelVisibility::Opaque,
            VoxelVisibility::Opaque => false,
        }
}

#[inline]
fn get_max_width<T>(
    voxels: &[T],
    visited: &[u8],
    merge_value: &T::MergeValue,
    merge_neighbor_value: &T::MergeValueFacingNeighbour,
    mut index: u32,
    n_stride: u32,
    u_stride: u32,
    max_width: u32,
) -> u32
where
    T: MergeVoxel,
{
    for width in 0..max_width {
        let voxel = unsafe { voxels.get_unchecked(index as usize) };
        let neighbor_index = index.wrapping_add(n_stride);
        let neighbor = unsafe { voxels.get_unchecked(neighbor_index as usize) };

        if !face_needs_mesh(visited, index, voxel, neighbor)
            || !voxel.merge_value().eq(merge_value)
            || !neighbor
                .merge_value_facing_neighbour()
                .eq(merge_neighbor_value)
        {
            return width;
        }

        index += u_stride;
    }

    max_width
}

#[inline]
fn get_max_height<T>(
    voxels: &[T],
    visited: &[u8],
    merge_value: &T::MergeValue,
    merge_neighbor_value: &T::MergeValueFacingNeighbour,
    mut index: u32,
    n_stride: u32,
    u_stride: u32,
    v_stride: u32,
    width: u32,
    max_height: u32,
) -> u32
where
    T: MergeVoxel,
{
    for height in 1..max_height {
        let row_width = get_max_width(
            voxels,
            visited,
            merge_value,
            merge_neighbor_value,
            index,
            n_stride,
            u_stride,
            width,
        );

        if row_width < width {
            return height;
        }

        index = index.wrapping_add(v_stride);
    }

    max_height
}

#[inline]
fn mark_visited(
    visited: &mut [u8],
    quad: UnorientedQuad,
    minimum_index: u32,
    u_stride: u32,
    v_stride: u32,
    lod: usize,
) {
    for u in 0..quad.width {
        for v in 0..quad.height {
            let index = minimum_index + u_stride * u + v_stride * v;

            visited[index as usize] |= 1 << lod;
        }
    }
}

#[inline]
fn find_max_lod<const X: u32, const Y: u32, const Z: u32, const M: usize>(
    visited: &mut [u8],
    quad: UnorientedQuad,
    face: OrientedBlockFace,
    u_stride: u32,
    v_stride: u32,
) -> usize {
    let mut lod: usize = 0;

    for i in (1..M).rev() {
        let quad_lod = into_lod(quad, face, i);
        let index = ChunkShape::<X, Y, Z>::linearize(quad_lod.minimum);

        if !has_visited_lod(visited, quad_lod, index, u_stride, v_stride, i) {
            mark_visited(visited, quad_lod, index, u_stride, v_stride, i);

            if lod < i {
                lod = i;
            }
        }
    }

    lod
}

#[inline]
fn has_visited_lod(
    visited: &[u8],
    quad: UnorientedQuad,
    minimum_index: u32,
    u_stride: u32,
    v_stride: u32,
    lod: usize,
) -> bool {
    for i in 0..quad.width {
        for j in 0..quad.height {
            let index = minimum_index + u_stride * i + v_stride * j;

            if visited[index as usize] & (1 << lod) == 0 {
                return false;
            }
        }
    }

    true
}

#[inline]
fn into_lod(quad: UnorientedQuad, face: OrientedBlockFace, lod: usize) -> UnorientedQuad {
    let minimum = quad.minimum;
    let maximum = quad.minimum + quad.width * face.u + quad.height * face.v;

    let new_minimum = (((minimum - 1) >> (lod as u32)) << (lod as u32)) + 1;
    let new_maximum = (((maximum - 1 + (1u32 << (lod as u32)).saturating_sub(1)) >> (lod as u32))
        << (lod as u32))
        + 1;

    let size = new_maximum - new_minimum;

    UnorientedQuad {
        minimum: new_minimum,
        width: size.dot(face.u),
        height: size.dot(face.v),
    }
}

// pub fn extract_greedy_lod_buffer<const M: usize>(
//     greedy_buffer: &PopBuffer<M, UnorientedQuad>,
//     output_buffer: &mut QuadBuffer<UnorientedQuad>,
//     lod: usize,
// ) {
//     for (face_index, group) in greedy_buffer.groups.iter().enumerate() {
//         let face = OrientedBlockFace::FACES[face_index];
//         let to_len = group.buckets[lod];
//         output_buffer.groups[face_index].extend(
//             group.quads[0..to_len]
//                 .iter()
//                 .map(|&quad| into_lod(quad, face, lod)),
//         );
//     }
// }
