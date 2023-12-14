#define_import_path bevy_mesh_pop::lod_functions

#import bevy_pbr::mesh_functions as       mesh_functions
#import bevy_pbr::mesh_view_bindings      view
#import bevy_pbr::mesh_bindings           mesh
#import bevy_mesh_pop::lod_bindings       size, max_lod, period, buckets

fn position_into_lod(index: u32, position: vec3<f32>, normal: vec3<f32>, lod: u32) -> vec3<f32> {
    let face = get_face(normal);
    let position = vec3<u32>(position);

    var n = dot(face.n_axis, position);
    var u = dot(face.u_axis, position);
    var v = dot(face.v_axis, position);

    // flatten the face
    if face.n_sign > 0 {
        n -= 1u;
    }

    // round to LOD
    n = into_lod_min(n, lod);

    if index % 4u == 0u {
        u = into_lod_min(u, lod);
        v = into_lod_min(v, lod);
    } else if index % 4u == 1u {
        u = into_lod_max(u, lod);
        v = into_lod_min(v, lod);
    } else if index % 4u == 2u {
        u = into_lod_min(u, lod);
        v = into_lod_max(v, lod);
    } else {
        u = into_lod_max(u, lod);
        v = into_lod_max(v, lod);
    }

    // raise face to LOD
    if face.n_sign > 0 {
        n += 1u << lod;
    }

    // reconstruct position
    let n_vec = vec3<f32>(face.n_axis * n);
    let u_vec = vec3<f32>(face.u_axis * u);
    let v_vec = vec3<f32>(face.v_axis * v);

    return n_vec + u_vec + v_vec;
}

fn position_into_lod_minimum(index: u32, position: vec3<f32>, normal: vec3<f32>, lod: u32) -> vec3<f32> {
    let face = get_face(normal);
    let position = vec3<u32>(position);

    var n = dot(face.n_axis, position);
    var u = dot(face.u_axis, position);
    var v = dot(face.v_axis, position);

    // flatten the face
    if face.n_sign > 0 {
        n -= 1u;
    }

    // round to LOD
    n = into_lod_min(n, lod);

    if index % 4u == 0u {
        u = into_lod_min(u, lod);
        v = into_lod_min(v, lod);
    } else if index % 4u == 1u {
        u = into_lod_min(u, lod);
        v = into_lod_min(v, lod);
    } else if index % 4u == 2u {
        u = into_lod_min(u, lod);
        v = into_lod_min(v, lod);
    } else {
        u = into_lod_min(u, lod);
        v = into_lod_min(v, lod);
    }

    // raise face to LOD
    if face.n_sign > 0 {
        n += 1u << lod;
    }

    // reconstruct position
    let n_vec = vec3<f32>(face.n_axis * n);
    let u_vec = vec3<f32>(face.u_axis * u);
    let v_vec = vec3<f32>(face.v_axis * v);

    return n_vec + u_vec + v_vec;
}

fn position_into_minimum(index: u32, position: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    let face = get_face(normal);
    var position = vec3<f32>(position);

    if index % 4u == 0u {
        // okay
    } else if index % 4u == 1u {
        position -= vec3<f32>(face.u_axis);
    } else if index % 4u == 2u {
        position -= vec3<f32>(face.v_axis);
    } else {
        position -= vec3<f32>(face.u_axis);
        position -= vec3<f32>(face.v_axis);
    }

    return position;
}

fn into_lod_min(position: u32, lod: u32) -> u32 {
    return (((position - 1u) >> lod) << lod) + 1u;
}

fn into_lod_max(position: u32, lod: u32) -> u32 {
    return (((position - 2u + (1u << lod)) >> lod) << lod) + 1u;
}

struct Face {
    n_sign: i32,
    n_axis: vec3<u32>,
    u_axis: vec3<u32>,
    v_axis: vec3<u32>,
}

fn get_face(normal: vec3<f32>) -> Face {
    let abs_normal = vec3<u32>(abs(normal));
    var face: Face;

    face.n_sign = i32(sign(dot(normal, abs(normal))));
    face.n_axis = abs_normal;

    if all(abs_normal == vec3<u32>(1u, 0u, 0u)) {
        face.u_axis = vec3<u32>(0u, 0u, 1u);
        face.v_axis = vec3<u32>(0u, 1u, 0u);
    } else if all(abs_normal == vec3<u32>(0u, 1u, 0u)) {
        face.u_axis = vec3<u32>(0u, 0u, 1u);
        face.v_axis = vec3<u32>(1u, 0u, 0u);
    } else if all(abs_normal == vec3<u32>(0u, 0u, 1u)) {
        face.u_axis = vec3<u32>(1u, 0u, 0u);
        face.v_axis = vec3<u32>(0u, 1u, 0u);
    }

  return face;
}

fn mesh_distance() -> f32 {
    let world_position = mesh_functions::mesh_position_local_to_world(mesh.model, vec4<f32>(vec3<f32>(size) / 2.0, 1.0));

    return length(world_position.xyz - view.world_position);
}

const pi = 3.14159265358979323846264338327950288;

fn calculate_lod() -> f32 {
    let distance = mesh_distance();
    let clamped_distance = clamp(distance, 0.0, f32(period));

#ifdef EASING_LINEAR
    return f32(max_lod) / f32(period) * clamped_distance);
#endif

#ifdef EASING_QUADRATIC
    return f32(max_lod) * pow(clamoed_distance / f32(period), 2.0);
#endif

#ifdef EASING_CUBIC
    return f32(max_lod) * pow(clamped_distance / f32(period), 3.0);
#endif

#ifdef EASING_SINE
    return f32(max_lod) - f32(max_lod) * cos(pi * clamped_distance / (2.0 * f32(period)));
#endif
}

fn lod_index(lod: u32) -> u32 {
    return buckets[lod / 4u][lod % 4u];
}
