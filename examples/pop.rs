use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    window::PrimaryWindow,
};
use bevy_dolly::{
    prelude::{DollyCursorGrab, Fpv, Rig},
    system::Dolly,
};
use block_mesh_pop::{
    visible_faces_quads, ChunkShape, LodEasing, LodMaterial, LodMaterialPlugin, LodRenderPlugin,
    MeshVoxel, PopBuffer, VisitedBuffer, VoxelVisibility, WrappedMaterial,
};

#[derive(Component)]
pub struct MainCamera;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            LodRenderPlugin,
            LodMaterialPlugin::<5, StandardMaterial>::default(),
            DollyCursorGrab,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (Dolly::<MainCamera>::update_active, update_camera))
        .run();
}

fn setup(
    mut commands: Commands,
    mut lod_materials: ResMut<Assets<LodMaterial<5>>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(48.0, 48.0, 48.0),
        point_light: PointLight {
            range: 200.0,
            intensity: 80000.0,
            ..default()
        },
        ..default()
    });

    let transform = Transform::from_xyz(48.0, 48.0, 48.0).looking_at(Vec3::splat(17.0), Vec3::Y);

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
    let (buckets, mesh) = generate_visible_faces_mesh(&voxels);

    commands.spawn((
        meshes.add(mesh),
        SpatialBundle::INHERITED_IDENTITY,
        lod_materials.add(LodMaterial {
            size: UVec3::splat(32),
            max_lod: 5,
            period: 128 / 2,
            easing: LodEasing::Quadratic,
            buckets: unsafe { std::mem::transmute(buckets) },
        }),
        WrappedMaterial(materials.add(StandardMaterial::from(Color::PURPLE))),
    ));
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

fn generate_visible_faces_mesh(voxels: &[Voxel]) -> ([u32; 8], Mesh) {
    let mut visited = VisitedBuffer::new(voxels.len());
    let mut buffer = PopBuffer::new();

    visible_faces_quads::<34, 34, 34, 5, _>(voxels, &mut visited, &mut buffer);

    let buckets = buffer.get_buckets();

    let num_quads = buffer.num_quads();

    let num_indices = num_quads * 6;
    let num_vertices = num_quads * 4;

    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut colors = Vec::with_capacity(num_vertices);

    for (face, quad) in buffer.iter_quads() {
        indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
        positions.extend_from_slice(&face.quad_mesh_positions(quad, 0, 1.0));
        normals.extend_from_slice(&face.quad_mesh_normals());
        colors.extend_from_slice(&[Vec4::ONE; 4]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![Vec2::ZERO; num_vertices]);
    mesh.set_indices(Some(Indices::U32(indices)));

    (buckets, mesh)
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
