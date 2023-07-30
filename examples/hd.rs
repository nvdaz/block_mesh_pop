use bevy::{
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_voxel_mesh::{
    extract_lod_buffer, visible_faces_quads, ChunkShape, MeshVoxel, PopBuffer, QuadsBuffer,
    UnorientedUnitQuad, VoxelVisibility,
};

#[derive(Component)]
struct Chunk {
    pop_buffer: PopBuffer<7, UnorientedUnitQuad>,
    lod: usize,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, WireframePlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, handle_lod_change))
        .run();
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(155.0, 155.0, 155.0),
        point_light: PointLight {
            range: 200.0,
            intensity: 8000.0,
            ..Default::default()
        },
        ..Default::default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(240.0, 240.0, 240.0).looking_at(Vec3::splat(65.0), Vec3::Y),
        ..Default::default()
    });

    let voxels = generate_voxels();
    let pop_buffer = generate_visible_faces_pop_buffer(&voxels);
    let material = StandardMaterial::from(Color::BLACK);

    commands
        .spawn(PbrBundle {
            material: materials.add(material),
            ..default()
        })
        .insert(Wireframe)
        .insert(Chunk { pop_buffer, lod: 0 });
}

fn handle_input(keys: Res<Input<KeyCode>>, mut chunk_q: Query<&mut Chunk>) {
    if keys.just_pressed(KeyCode::L) {
        let mut chunk = chunk_q.single_mut();

        chunk.lod = (chunk.lod + 1) % 7;
    }
    if keys.just_pressed(KeyCode::K) {
        let mut chunk = chunk_q.single_mut();

        chunk.lod = (chunk.lod + 6) % 7;
    }
}

fn handle_lod_change(
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_q: Query<(&mut Handle<Mesh>, &Chunk), Changed<Chunk>>,
) {
    if let Ok((mut mesh_handle, chunk)) = chunk_q.get_single_mut() {
        let mesh = generate_visible_faces_mesh(&chunk.pop_buffer, chunk.lod);

        *mesh_handle = meshes.add(mesh);
    }
}

fn generate_voxels() -> Vec<Voxel> {
    let mut voxels = vec![Voxel::EMPTY; 130 * 130 * 130];

    for i in 0..voxels.len() {
        let position = ChunkShape::<130, 130, 130>::delinearize(i as u32);
        let position = position.as_vec3();
        let center = Vec3::splat(65.0);

        if position.distance_squared(center) < 64.0 * 64.0 {
            voxels[i] = Voxel::FULL;
        }
    }

    voxels
}

fn generate_visible_faces_pop_buffer(voxels: &[Voxel]) -> PopBuffer<7, UnorientedUnitQuad> {
    let mut pop_buffer = PopBuffer::new();
    visible_faces_quads::<130, 130, 130, 7, _>(voxels, &mut pop_buffer);

    pop_buffer
}

fn generate_visible_faces_mesh(pop_buffer: &PopBuffer<7, UnorientedUnitQuad>, lod: usize) -> Mesh {
    let mut buffer = QuadsBuffer::new();

    extract_lod_buffer(pop_buffer, &mut buffer, lod);

    let num_indices = buffer.num_quads() * 6;
    let num_vertices = buffer.num_quads() * 4;

    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);

    for (face, quad) in buffer.iter_quads() {
        indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
        positions.extend_from_slice(&face.quad_mesh_positions(quad, lod, 1.0));
        normals.extend_from_slice(&face.quad_mesh_normals());
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![Vec2::ZERO; num_vertices]);
    mesh.set_indices(Some(Indices::U32(indices)));

    mesh
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Voxel {
    EMPTY,
    FULL,
}
impl MeshVoxel for Voxel {
    fn get_visibility(&self) -> VoxelVisibility {
        match *self {
            Self::EMPTY => VoxelVisibility::Empty,
            Self::FULL => VoxelVisibility::Opaque,
        }
    }
}
