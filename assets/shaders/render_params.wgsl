const N_SHADOWS: i32 = 4;

struct RenderParams {
    proj: mat4x4<f32>,
    invproj: mat4x4<f32>,
    sunproj: array<mat4x4<f32>, N_SHADOWS>,
    cam_pos: vec4<f32>,
    cam_dir: vec4<f32>,
    sun: vec3<f32>,
    sun_col: vec4<f32>,
    sand_col: vec4<f32>,
    sea_col: vec4<f32>,
    viewport: vec2<f32>,
    unproj_pos: vec2<f32>,
    time: f32,
    time_always: f32,
    shadow_mapping_resolution: i32,
    terraforming_mode_radius: f32,
}