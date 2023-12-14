#import bevy_pbr::mesh_functions as       mesh_functions
#import bevy_pbr::mesh_bindings           mesh
#import bevy_pbr::mesh_view_bindings      view
#import bevy_pbr::mesh_vertex_output      MeshVertexOutput
#import bevy_mesh_pop::lod_functions as   lod_functions
#import bevy_mesh_pop::lod_bindings       max_lod, size, period


struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @builtin(vertex_index) index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
#ifdef VERTEX_TANGENTS
    @location(3) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
#endif
}

@vertex
fn vertex(vertex: Vertex) -> MeshVertexOutput {

    let lod = lod_functions::calculate_lod();

    var position: vec3<f32>;

    if (lod % 1.0) <= 0.25 && lod >= 1.0 {
        let floor_lod = u32(floor(lod));
        let ceil_lod = u32(ceil(lod));

        let is_next = vertex.index < lod_functions::lod_index(floor_lod) * 6u;

        var current_position = lod_functions::position_into_lod(vertex.index, vertex.position, vertex.normal, floor_lod);

        var next_position: vec3<f32>;

        next_position = lod_functions::position_into_lod(vertex.index, vertex.position, vertex.normal, floor_lod - 1u);

        let face = lod_functions::get_face(vertex.normal);
        if is_next {
            position = mix(current_position, next_position, 1.0 - (lod % 1.0) / 0.25);

        } else {
            let distance = lod_functions::mesh_distance();
            current_position -= f32(face.n_sign) * vec3<f32>(face.n_axis) * clamp(lod % 1.0, 0.1, 0.25) * (f32(((vertex.index / 4u) % 4u) + 1u) * 2.0) / 100.0;
            position = mix(current_position, next_position, 1.0 - (lod % 1.0) / 0.25);
        }



    } else {
        let floor_lod = u32(floor(lod));

        position = lod_functions::position_into_lod(vertex.index, vertex.position, vertex.normal, u32(floor(lod)));
    }


    // let lod = u32(floor(lod_functions::calculate_lod()));
    // let position = lod_functions::position_into_lod(vertex.index, vertex.position, vertex.normal, lod);

    var out: MeshVertexOutput;

    out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal);
    out.world_position = mesh_functions::mesh_position_local_to_world(mesh.model, vec4<f32>(vec3<f32>(position), 1.0));
    out.position = mesh_functions::mesh_position_world_to_clip(out.world_position);
    out.uv = vertex.uv;

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        model,
        vertex.tangent,
        vertex.instance_index
    );
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

    return out;
}
