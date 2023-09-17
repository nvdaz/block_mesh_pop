#define_import_path bevy_mesh_pop::lod_bindings

@group(3) @binding(0)
var<uniform> size: vec3<u32>;

@group(3) @binding(1)
var<uniform> max_lod: u32;

@group(3) @binding(2)
var<uniform> period: u32;

@group(3) @binding(3)
var<uniform> buckets: array<vec4<u32>, 2>;
