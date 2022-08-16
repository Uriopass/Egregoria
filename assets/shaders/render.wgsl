#include "dither.wgsl"

fn render(sun: vec3<f32>,
          cam: vec3<f32>,
          wpos: vec3<f32>,
          position: vec2<f32>,
          normal: vec3<f32>,
          base_color: vec3<f32>,
          sun_col: vec3<f32>,
          shadow_v: f32,
          ssao: f32) -> vec3<f32>  {
    let R: vec3<f32> = normalize(2.0 * normal * dot(normal,sun) - sun);
    let V: vec3<f32> = normalize(cam - wpos);

    var specular: f32 = clamp(dot(R, V), 0.0, 1.0);
    specular = pow(specular, 5.0);

    let sun_contrib: f32 = clamp(dot(normal, sun), 0.0, 1.0);

    let ambiant: vec3<f32> = 0.15 * base_color;
    let sunpower: f32 = (0.85 * sun_contrib + 0.5 * specular) * shadow_v;

    var final_rgb: vec3<f32> = ambiant + sunpower * (sun_col * base_color);
    final_rgb = final_rgb * ssao;
    final_rgb = final_rgb + dither(position);
    return final_rgb;
}