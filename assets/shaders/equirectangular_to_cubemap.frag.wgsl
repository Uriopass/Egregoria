struct FragmentOutput {
    @location(0) out_cube: vec4<f32>,
}

// equirectangular tex (rg16f is not filterable)
@group(0) @binding(0) var t: texture_2d<f32>;

const PI: f32 = 3.141592653589793238462;
const TAU: f32 = 6.283185307179586476925;

fn cartesian_to_spherical(v: vec3<f32>) -> vec2<f32> {
    let r: f32 = length(v);
    var theta: f32 = acos(v.z / r) / PI;
    var phi: f32 = -atan2(v.y, v.x) / TAU + 0.5;

    return vec2<f32>(phi, theta);
}

@fragment
fn frag(@location(0) wpos: vec3<f32>) -> FragmentOutput {
    let uv1 = cartesian_to_spherical(wpos);
    let dim: vec2<i32> = textureDimensions(t, 0);
    let uv: vec2<i32> = vec2<i32>(i32(uv1.x * f32(dim.x)) % dim.x, i32(uv1.y * f32(dim.y)) % dim.y);
    let v: vec4<f32> = textureLoad(t, uv, 0);
    return FragmentOutput(v);
}