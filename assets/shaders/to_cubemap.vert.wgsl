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


@vertex
fn vert(@builtin(vertex_index) index: u32) -> VertexOutput {
    let index6 = index % 6u;
    let in_pos: vec2<f32> = poses[index6];
    let in_wpos: vec3<f32> = uvs[index / 6u * 4u  + index6 % 3u + index6 / 4u ];

    return VertexOutput(in_wpos, vec4<f32>(in_pos.x, in_pos.y, 0.0, 1.0));
}