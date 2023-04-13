const N_SHADOWS: i32 = 4;

struct RenderParams {
    invproj: mat4x4<f32>,
    sunproj: array<mat4x4<f32>, N_SHADOWS>,
    cam_pos: vec4<f32>,
    cam_dir: vec4<f32>,
    sun: vec3<f32>,
    sun_col: vec4<f32>,
    grass_col: vec4<f32>,
    sand_col: vec4<f32>,
    sea_col: vec4<f32>,
    viewport: vec2<f32>,
    time: f32,
    time_always: f32,
    ssao_enabled: i32,
    shadow_mapping_resolution: i32,
    realistic_sky: i32,
    grid_enabled: i32,
}