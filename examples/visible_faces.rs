use bevy::{
    input::mouse::MouseMotion,
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    window::PrimaryWindow,
};
use bevy_dolly::{
    prelude::{DollyCursorGrab, Fpv, Rig},
    system::Dolly,
};
use bevy_voxel_mesh::{
    extract_lod_buffer, visible_faces_quads, ChunkShape, MeshVoxel, PopBuffer, QuadsBuffer,
    VoxelVisibility,
};

#[derive(Component)]
pub struct MainCamera;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, WireframePlugin, DollyCursorGrab))
        .add_systems(Startup, setup)
        .add_systems(Update, (Dolly::<MainCamera>::update_active, update_camera))
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(25.0, 25.0, 25.0),
        point_light: PointLight {
            range: 200.0,
            intensity: 8000.0,
            ..default()
        },
        ..default()
    });

    let transform = Transform::from_xyz(-48.0, 48.0, -48.0).looking_at(Vec3::splat(17.0), Vec3::Y);

    commands.spawn((
        MainCamera,
        Rig::builder()
            .with(Fpv::from_position_target(transform))
            .build(),
        Camera3dBundle {
            transform,
            ..default()
        },
    ));

    let voxels = generate_voxels();
    let mesh = generate_visible_faces_mesh(&voxels);
    let material = StandardMaterial::from(Color::BLACK);

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(material),
            ..default()
        })
        .insert(Wireframe);
}

fn generate_voxels() -> [Voxel; 34 * 34 * 34] {
    let mut voxels = [Voxel::EMPTY; 34 * 34 * 34];

    for i in 0..voxels.len() {
        let position = ChunkShape::<34, 34, 34>::delinearize(i as u32);
        let position = position.as_vec3();
        let center = Vec3::splat(17.0);

        if position.distance_squared(center) < 12.0 * 12.0 {
            voxels[i] = Voxel::FULL;
        }
    }

    voxels
}

fn generate_visible_faces_mesh(voxels: &[Voxel]) -> Mesh {
    let mut pop_buffer = PopBuffer::new();
    let mut buffer = QuadsBuffer::new();

    visible_faces_quads::<34, 34, 34, 4, _>(voxels, &mut pop_buffer);

    let lod = 1;
    extract_lod_buffer(&pop_buffer, &mut buffer, lod);

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

fn update_camera(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut rig_q: Query<&mut Rig>,
) {
    let time_delta_seconds: f32 = time.delta_seconds();
    let boost_mult = 5.0f32;
    let sensitivity = Vec2::splat(1.0);

    let mut move_vec = Vec3::ZERO;

    if keys.pressed(KeyCode::W) {
        move_vec.z -= 1.0;
    }
    if keys.pressed(KeyCode::S) {
        move_vec.z += 1.0;
    }
    if keys.pressed(KeyCode::A) {
        move_vec.x -= 1.0;
    }
    if keys.pressed(KeyCode::D) {
        move_vec.x += 1.0;
    }

    if keys.pressed(KeyCode::E) || keys.pressed(KeyCode::Space) {
        move_vec.y += 1.0;
    }
    if keys.pressed(KeyCode::Q) {
        move_vec.y -= 1.0;
    }

    let boost: f32 = if keys.pressed(KeyCode::ShiftLeft) {
        boost_mult
    } else {
        1.
    };

    let mut delta = Vec2::ZERO;
    for event in mouse_motion_events.iter() {
        delta += event.delta;
    }
    delta.x *= sensitivity.x;
    delta.y *= sensitivity.y;

    let mut rig = rig_q.single_mut();

    if let Ok(window) = windows.get_single() {
        if !window.cursor.visible {
            rig.driver_mut::<Fpv>().update_pos_rot(
                move_vec,
                delta,
                true,
                boost,
                time_delta_seconds,
            );
        }
    }
}
