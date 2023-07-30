// use std::{cmp, collections::HashSet};

// use bevy_math::UVec3;

// use crate::{
//     buffer::UnitQuadsBuffer,
//     geometry::{
//         face::OrientedBlockFace,
//         quad::{UnorientedQuad, UnorientedUnitQuad},
//         shape::ChunkShape,
//     },
//     MergeVoxel, QuadsBuffer, VoxelVisibility,
// };

// pub fn visible_faces_quads<const X: u32, const Y: u32, const Z: u32, V: MergeVoxel>(
//     voxels: &[V],
//     output: &mut UnitQuadsBuffer,
// ) {
//     let strides = OrientedBlockFace::FACES
//         .map(|face| ChunkShape::<X, Y, Z>::linearize(face.signed_normal().as_uvec3()));

//     for position in ChunkShape::<X, Y, Z>::inner_iter() {
//         let index = ChunkShape::<X, Y, Z>::linearize(position);
//         let voxel = unsafe { voxels.get_unchecked(index as usize) };

//         if voxel.get_visibility() != VoxelVisibility::Empty {
//             for (face_index, &face_stride) in strides.iter().enumerate() {
//                 let neighbor_index = index.wrapping_add(face_stride);
//                 let neighbor_voxel = unsafe { voxels.get_unchecked(neighbor_index as usize) };

//                 let face_needs_mesh = match neighbor_voxel.get_visibility() {
//                     VoxelVisibility::Empty => true,
//                     VoxelVisibility::Translucent => {
//                         voxel.get_visibility() == VoxelVisibility::Opaque
//                     }
//                     VoxelVisibility::Opaque => false,
//                 };

//                 if face_needs_mesh {
//                     output.faces[face_index].push(UnorientedUnitQuad { minimum: position })
//                 }
//             }
//         }
//     }
// }

// fn into_lod<const M: usize>(quad: UnorientedUnitQuad, lod: usize) -> UnorientedQuad {
//     UnorientedQuad {
//         minimum: (quad.minimum >> ((M - lod) as u32)) << ((M - lod) as u32),
//         size: UVec3::splat(1 << (M - lod)),
//     }
// }

// fn max_lod<const M: usize>(quad: UnorientedQuad, face: OrientedBlockFace) -> usize {
//     let u_pos = quad.minimum.dot(face.u);
//     let v_pos = quad.minimum.dot(face.v);

//     (M - 1).saturating_sub(cmp::max(u_pos.trailing_zeros(), v_pos.trailing_zeros()) as usize)
// }

// pub fn create_lod_buffer<const M: usize>(quads: &mut UnitQuadsBuffer) -> [[usize; M]; 6] {
//     let mut buckets = [[0; M]; 6];

//     for (face_idx, (face, group)) in quads.iter_faces_mut().enumerate() {
//         // count quads for each bucket
//         for quad in group.iter() {
//             let lod = max_lod::<M>((*quad).into(), face);
//             for i in lod..M {
//                 buckets[face_idx][i] += 1;
//             }
//         }

//         // sort quads
//         for i in (0..group.len()).rev() {
//             loop {
//                 let quad = group[i];
//                 let lod = max_lod::<M>(quad.into(), face);
//                 let ptr = buckets[face_idx][lod];
//                 if i < ptr {
//                     break;
//                 }

//                 group.swap(i, ptr);
//                 buckets[face_idx][lod] += 1;
//             }
//         }

//         // dedupe quads in each bucket
//         for lod in 1..M {
//             let mut positions = HashSet::new();
//             let prev_len = if lod > 1 {
//                 buckets[face_idx][lod - 2]
//             } else {
//                 0
//             };
//             let len = &mut buckets[face_idx][lod - 1];

//             for i in (0..prev_len).chain((prev_len..*len).rev()) {
//                 let into_lod_quad = into_lod::<M>(group[i], lod);
//                 if !positions.insert(into_lod_quad.minimum.to_array()) {
//                     group.swap(i, *len - 1);

//                     *len -= 1;
//                 }
//             }
//         }
//     }

//     buckets
// }

// pub fn extract_lod_buffer<const M: usize>(
//     quads: &UnitQuadsBuffer,
//     buckets: [[usize; M]; 6],
//     target_lod: usize,
// ) -> QuadsBuffer {
//     debug_assert!(target_lod <= M);

//     let mut buffer = QuadsBuffer::new();

//     for (face_idx, bucket) in buckets.iter().enumerate() {
//         let to_len = bucket[target_lod - 1];

//         buffer.faces[face_idx].extend(
//             quads.faces[face_idx][0..to_len]
//                 .iter()
//                 .map(|quad| into_lod::<M>(*quad, target_lod)),
//         );
//     }

//     buffer
// }

// // pub fn generate_visible_faces_mesh<V: MergeVoxel, S: Shape<3, Coord = u32>>(
// //     voxels: &[V],
// //     voxels_shape: S,
// //     min: UVec3,
// //     max: UVec3,
// //     buffer: &mut UnitQuadBuffer,
// // ) -> Mesh {
// //     visibile_faces_quads(voxels, voxels_shape, min, max, buffer);

// //     let num_indices = buffer.num_quads() * 6;
// //     let num_vertices = buffer.num_quads() * 4;
// //     let mut indices = Vec::with_capacity(num_indices);
// //     let mut positions = Vec::with_capacity(num_vertices);
// //     let mut normals = Vec::with_capacity(num_vertices);

// //     for (group, face) in buffer.groups.iter().zip(OrientedBlockFace::get_faces()) {
// //         for quad in group.iter() {
// //             indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32))
// //         }
// //     }

// //     let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

// //     mesh
// // }
