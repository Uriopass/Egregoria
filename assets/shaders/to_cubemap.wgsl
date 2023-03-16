struct VertexOutput {
    @location(0) wpos: vec3<f32>,
    @builtin(position) out_pos: vec4<f32>,
}

var<private> poses: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, 1.0),
);

var<private> uvs: array<vec3<f32>, 24> = array<vec3<f32>, 24>(
    // X+
    vec3<f32>(1.0, -1.0, 1.0),
    vec3<f32>(1.0, -1.0, -1.0),
    vec3<f32>(1.0, 1.0, -1.0),
    vec3<f32>(1.0, 1.0, 1.0),

    // X-
    vec3<f32>(-1.0, -1.0, -1.0),
    vec3<f32>(-1.0, -1.0, 1.0),
    vec3<f32>(-1.0, 1.0, 1.0),
    vec3<f32>(-1.0, 1.0, -1.0),

    // Y+
    vec3<f32>(-1.0, 1.0, 1.0),
    vec3<f32>(1.0,  1.0, 1.0),
    vec3<f32>(1.0,  1.0, -1.0),
    vec3<f32>(-1.0, 1.0, -1.0),

    // Y-
    vec3<f32>(-1.0, -1.0, -1.0),
    vec3<f32>(1.0, -1.0, -1.0),
    vec3<f32>(1.0, -1.0, 1.0),
    vec3<f32>(-1.0, -1.0, 1.0),

    // Z+
    vec3<f32>(-1.0, -1.0, 1.0),
    vec3<f32>(1.0, -1.0, 1.0),
    vec3<f32>(1.0, 1.0, 1.0),
    vec3<f32>(-1.0, 1.0, 1.0),

    // Z-
    vec3<f32>(1.0, -1.0, -1.0),
    vec3<f32>(-1.0, -1.0, -1.0),
    vec3<f32>(-1.0, 1.0, -1.0),
    vec3<f32>(1.0, 1.0, -1.0)
);

const HALF_PI: f32 = 1.570796326794896619231;
const PI: f32 = 3.141592653589793238462;
const TAU: f32 = 6.283185307179586476925;

fn cartesian_to_spherical(v: vec3<f32>) -> vec2<f32> {
    let r: f32 = length(v);
    var theta: f32 = acos(v.z / r) / PI;
    var phi: f32 = atan2(v.y, v.x) / TAU + 0.5;

    return vec2<f32>(phi, theta);
}

@vertex
fn vert(@builtin(vertex_index) index: u32) -> VertexOutput {
    let index6 = index % 6u;
    let in_pos: vec2<f32> = poses[index6];
    let in_wpos: vec3<f32> = uvs[index / 6u * 4u  + index6 % 3u + index6 / 4u ];

    return VertexOutput(in_wpos, vec4<f32>(in_pos.x, in_pos.y, 0.0, 1.0));
}

struct FragmentOutput {
    @location(0) out_cube: vec4<f32>,
}

@group(0) @binding(0) var t: texture_2d<f32>;

@fragment
fn frag(@builtin(position) pos: vec4<f32>, @location(0) wpos: vec3<f32>) -> FragmentOutput {
    let uv1 = cartesian_to_spherical(wpos);
    let dim: vec2<i32> = textureDimensions(t, 0);
    let uv: vec2<i32> = vec2<i32>(i32(uv1.x * f32(dim.x)) % dim.x, i32(uv1.y * f32(dim.y)) % dim.y);
    let v: vec4<f32> = textureLoad(t, uv, 0);
    return FragmentOutput(v);
}