use bevy_math::Vec3;
use bevy_voxel_mesh::{
    visible_greedy_quads, ChunkShape, GreedyVisitedBuffer, MergeVoxel, MeshVoxel, PopBuffer,
    VoxelVisibility,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

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

impl MergeVoxel for Voxel {
    type MergeValue = Self;
    type MergeValueFacingNeighbour = Self;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }

    fn merge_value_facing_neighbour(&self) -> Self::MergeValueFacingNeighbour {
        *self
    }
}

pub fn empty_mesh(c: &mut Criterion) {
    let voxels = [Voxel::EMPTY; 66 * 66 * 66];
    let mut pop_buffer = PopBuffer::new();
    let mut visited_buffer = GreedyVisitedBuffer::new(voxels.len());

    c.bench_function("greedy empty mesh", |b| {
        b.iter(|| {
            pop_buffer.reset();
            visible_greedy_quads::<66, 66, 66, 1, _>(&voxels, &mut pop_buffer, &mut visited_buffer);
            black_box(&pop_buffer);
        })
    });
}

pub fn sphere_mesh(c: &mut Criterion) {
    let mut voxels = [Voxel::EMPTY; 66 * 66 * 66];

    let mut pop_buffer = PopBuffer::new();
    let mut visited_buffer = GreedyVisitedBuffer::new(voxels.len());
    for i in 0..voxels.len() {
        let position = ChunkShape::<66, 66, 66>::delinearize(i as u32);
        let position = position.as_vec3();
        let center = Vec3::splat(33.0);

        if position.distance_squared(center) < 32.0 * 32.0 {
            voxels[i] = Voxel::FULL;
        }
    }

    c.bench_function("greedy sphere mesh", |b| {
        b.iter(|| {
            pop_buffer.reset();
            visible_greedy_quads::<66, 66, 66, 1, _>(&voxels, &mut pop_buffer, &mut visited_buffer);
            black_box(&pop_buffer);
        })
    });
}

// pub fn sphere_mesh_lod(c: &mut Criterion) {
//     let mut voxels = [Voxel::EMPTY; 66 * 66 * 66];
//     let mut buffer = UnitQuadsPopBuffer::new();

//     for i in 0..voxels.len() {
//         let position = ChunkShape::<66, 66, 66>::delinearize(i as u32);
//         let position = position.as_vec3();
//         let center = Vec3::splat(33.0);

//         if position.distance_squared(center) < 32.0 * 32.0 {
//             voxels[i] = Voxel::FULL;
//         }
//     }

//     c.bench_function("sphere mesh lod", |b| {
//         b.iter(|| {
//             buffer.reset();
//             visible_faces_quads::<66, 66, 66, 6, _>(&voxels, &mut buffer);
//             black_box(&buffer);
//         })
//     });
// }

criterion_group!(benches, empty_mesh, sphere_mesh);
criterion_main!(benches);
