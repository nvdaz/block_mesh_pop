use bevy_math::Vec3;
use bevy_voxel_mesh::{
    extract_lod_buffer, visible_faces_quads, ChunkShape, MeshVoxel, PopBuffer, QuadsBuffer,
    VoxelVisibility,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Voxel {
    EMPTY,
    FULL,
}

impl MeshVoxel for Voxel {
    fn get_visibility(&self) -> VoxelVisibility {
        match self {
            Self::EMPTY => VoxelVisibility::Empty,
            Self::FULL => VoxelVisibility::Opaque,
        }
    }
}

fn main() {
    let mut voxels = [Voxel::EMPTY; 66 * 66 * 66];
    let mut buffer = PopBuffer::new();

    for i in 0..voxels.len() {
        let position = ChunkShape::<66, 66, 66>::delinearize(i as u32);
        let position = position.as_vec3();
        let center = Vec3::splat(33.0);

        if position.distance_squared(center) < 32.0 * 32.0 {
            voxels[i] = Voxel::FULL;
        }
    }

    visible_faces_quads::<66, 66, 66, 6, _>(&voxels, &mut buffer);

    let mut quads = QuadsBuffer::new();

    extract_lod_buffer(&buffer, &mut quads, 3);
}
