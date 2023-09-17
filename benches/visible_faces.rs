use bevy_math::Vec3;
use block_mesh_pop::{
    visible_faces_quads, ChunkShape, MeshVoxel, PopBuffer, VisitedBuffer, VoxelVisibility,
};
use criterion::{criterion_group, criterion_main, Criterion};

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

pub fn empty_mesh(c: &mut Criterion) {
    let voxels = [Voxel::EMPTY; 66 * 66 * 66];
    let mut visited = VisitedBuffer::new(voxels.len());
    let mut buffer = PopBuffer::new();

    c.bench_function("visible faces empty mesh", |b| {
        buffer.reset();
        b.iter(|| {
            visible_faces_quads::<66, 66, 66, 1, _>(&voxels, &mut visited, &mut buffer);
        })
    });
}

pub fn sphere_mesh(c: &mut Criterion) {
    let mut voxels = [Voxel::EMPTY; 66 * 66 * 66];
    let mut visited = VisitedBuffer::new(voxels.len());
    let mut buffer = PopBuffer::new();

    for i in 0..voxels.len() {
        let position = ChunkShape::<66, 66, 66>::delinearize(i as u32);
        let position = position.as_vec3();
        let center = Vec3::splat(33.0);

        if position.distance_squared(center) < 32.0 * 32.0 {
            voxels[i] = Voxel::FULL;
        }
    }

    c.bench_function("visible faces sphere mesh", |b| {
        buffer.reset();
        b.iter(|| {
            visible_faces_quads::<66, 66, 66, 1, _>(&voxels, &mut visited, &mut buffer);
        })
    });
}

pub fn sphere_mesh_lod(c: &mut Criterion) {
    let mut voxels = [Voxel::EMPTY; 66 * 66 * 66];
    let mut visited = VisitedBuffer::new(voxels.len());
    let mut buffer = PopBuffer::new();

    for i in 0..voxels.len() {
        let position = ChunkShape::<66, 66, 66>::delinearize(i as u32);
        let position = position.as_vec3();
        let center = Vec3::splat(33.0);

        if position.distance_squared(center) < 32.0 * 32.0 {
            voxels[i] = Voxel::FULL;
        }
    }

    c.bench_function("visible faces sphere mesh lod", |b| {
        buffer.reset();
        b.iter(|| {
            visible_faces_quads::<66, 66, 66, 6, _>(&voxels, &mut visited, &mut buffer);
        })
    });
}

pub fn empty_mesh_small(c: &mut Criterion) {
    let voxels = [Voxel::EMPTY; 18 * 18 * 18];
    let mut visited = VisitedBuffer::new(voxels.len());
    let mut buffer = PopBuffer::new();

    c.bench_function("visible faces empty mesh small", |b| {
        buffer.reset();
        b.iter(|| {
            visible_faces_quads::<18, 18, 18, 1, _>(&voxels, &mut visited, &mut buffer);
        })
    });
}

pub fn sphere_mesh_small(c: &mut Criterion) {
    let mut voxels = [Voxel::EMPTY; 18 * 18 * 18];
    let mut visited = VisitedBuffer::new(voxels.len());
    let mut buffer = PopBuffer::new();

    for i in 0..voxels.len() {
        let position = ChunkShape::<18, 18, 18>::delinearize(i as u32);
        let position = position.as_vec3();
        let center = Vec3::splat(9.0);

        if position.distance_squared(center) < 7.5 * 7.5 {
            voxels[i] = Voxel::FULL;
        }
    }

    c.bench_function("visible faces sphere mesh small", |b| {
        buffer.reset();
        b.iter(|| {
            visible_faces_quads::<18, 18, 18, 1, _>(&voxels, &mut visited, &mut buffer);
        })
    });
}

pub fn sphere_mesh_lod_small(c: &mut Criterion) {
    let mut voxels = [Voxel::EMPTY; 18 * 18 * 18];
    let mut visited = VisitedBuffer::new(voxels.len());
    let mut buffer = PopBuffer::new();

    for i in 0..voxels.len() {
        let position = ChunkShape::<18, 18, 18>::delinearize(i as u32);
        let position = position.as_vec3();
        let center = Vec3::splat(9.0);

        if position.distance_squared(center) < 7.5 * 7.5 {
            voxels[i] = Voxel::FULL;
        }
    }

    c.bench_function("visible faces sphere mesh lod small", |b| {
        buffer.reset();
        b.iter(|| {
            visible_faces_quads::<18, 18, 18, 4, _>(&voxels, &mut visited, &mut buffer);
        })
    });
}

criterion_group!(
    benches,
    empty_mesh,
    sphere_mesh,
    sphere_mesh_lod,
    empty_mesh_small,
    sphere_mesh_small,
    sphere_mesh_lod_small
);
criterion_main!(benches);
