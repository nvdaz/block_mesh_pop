use std::cmp;

use bevy_math::UVec3;

use crate::{
    geometry::face::OrientedBlockFace, ChunkShape, MergeVoxel, MeshVoxel, PopBuffer, QuadsBuffer,
    UnorientedQuad, VoxelVisibility,
};

pub struct GreedyVisitedBuffer {
    visited: Vec<bool>,
}

impl GreedyVisitedBuffer {
    #[inline]
    pub fn new(size: usize) -> Self {
        Self {
            visited: vec![false; size],
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.visited.fill(false);
    }
}

pub fn visible_greedy_quads<
    const X: u32,
    const Y: u32,
    const Z: u32,
    const M: usize,
    V: MergeVoxel,
>(
    voxels: &[V],
    pop_buffer: &mut PopBuffer<M, UnorientedQuad>,
    visited: &mut GreedyVisitedBuffer,
) {
    for (face_index, face) in OrientedBlockFace::FACES.into_iter().enumerate() {
        visited.reset();
        let interior_shape = UVec3::new(X - 2, Y - 2, Z - 2);

        let n_max = face.n.dot(interior_shape) + 1;
        let u_max = face.u.dot(interior_shape) + 1;
        let v_max = face.v.dot(interior_shape) + 1;
        let n_stride = ChunkShape::<X, Y, Z>::linearize(face.signed_normal().as_uvec3());
        let u_stride = ChunkShape::<X, Y, Z>::linearize(face.u);
        let v_stride = ChunkShape::<X, Y, Z>::linearize(face.v);

        for n in 1..n_max {
            for position in ChunkShape::<X, Y, Z>::slice_iter(face, n) {
                let index = ChunkShape::<X, Y, Z>::linearize(position);
                let voxel = unsafe { voxels.get_unchecked(index as usize) };

                let neighbor_index = index.wrapping_add(n_stride);
                let neighbor_voxel = unsafe { voxels.get_unchecked(neighbor_index as usize) };

                if face_needs_mesh(index, voxel, neighbor_voxel, &visited.visited) {
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

                    mark_visited::<X, Y, Z>(&mut visited.visited, quad, face);

                    let lod = max_lod::<M>(quad, face);

                    pop_buffer.add_quad(face_index, quad, lod)
                }
            }
        }
    }
}

#[inline]
fn face_needs_mesh<T>(index: u32, voxel: &T, neighbor: &T, visited: &[bool]) -> bool
where
    T: MeshVoxel,
{
    voxel.get_visibility() != VoxelVisibility::Empty
        && !visited[index as usize]
        && match neighbor.get_visibility() {
            VoxelVisibility::Empty => true,
            VoxelVisibility::Translucent => voxel.get_visibility() == VoxelVisibility::Opaque,
            VoxelVisibility::Opaque => false,
        }
}

#[inline]
fn get_max_width<T>(
    voxels: &[T],
    visited: &[bool],
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

        if !face_needs_mesh(index, voxel, neighbor, visited)
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
    visited: &[bool],
    merge_value: &T::MergeValue,
    neighbor_merge_value: &T::MergeValueFacingNeighbour,
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
            neighbor_merge_value,
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
fn mark_visited<const X: u32, const Y: u32, const Z: u32>(
    visited: &mut [bool],
    quad: UnorientedQuad,
    face: OrientedBlockFace,
) {
    for i in 0..quad.width {
        for j in 0..quad.height {
            let position_adj = quad.minimum + face.u * i + face.v * j;
            let index = ChunkShape::<X, Y, Z>::linearize(position_adj);

            visited[index as usize] = true;
        }
    }
}

#[inline]
fn max_lod<const M: usize>(quad: UnorientedQuad, face: OrientedBlockFace) -> usize {
    let u_pos = quad.minimum.dot(face.u);
    let v_pos = quad.minimum.dot(face.v);

    let u_max = u_pos + quad.width;
    let v_max = v_pos + quad.height;

    ((u32::BITS
        - cmp::max(
            (u_pos ^ u_max).leading_zeros(),
            (v_pos ^ v_max).leading_zeros(),
        )) as usize)
        .min(M - 1)
}

#[inline]
fn into_lod<const M: usize>(
    quad: UnorientedQuad,
    face: OrientedBlockFace,
    lod: usize,
) -> UnorientedQuad {
    let minimum = quad.minimum;
    let maximum = quad.minimum + face.u * quad.width + face.v * quad.height;

    let new_minimum = (minimum >> (lod as u32)) << (lod as u32);
    let new_maximum =
        ((maximum + (1u32 << (lod as u32)).saturating_sub(1)) >> (lod as u32)) << (lod as u32);

    let size = new_maximum - new_minimum;

    UnorientedQuad {
        minimum: new_minimum,
        width: size.dot(face.u),
        height: size.dot(face.v),
    }
}

pub fn extract_greedy_lod_buffer<const M: usize>(
    greedy_buffer: &PopBuffer<M, UnorientedQuad>,
    output_buffer: &mut QuadsBuffer<UnorientedQuad>,
    lod: usize,
) {
    for (face_indx, group) in greedy_buffer.groups.iter().enumerate() {
        let face = OrientedBlockFace::FACES[face_indx];
        let to_len = group.buckets[lod];
        output_buffer.groups[face_indx].extend(
            group.quads[0..to_len]
                .iter()
                .map(|&quad| into_lod::<M>(quad, face, lod)),
        );
    }
}
