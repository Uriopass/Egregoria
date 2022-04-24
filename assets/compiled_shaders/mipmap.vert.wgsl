struct VertexOutput {
    [[location(0)]] v_TexCoord: vec2<f32>;
    [[builtin(position)]] member: vec4<f32>;
};

var<private> v_TexCoord: vec2<f32>;
var<private> gl_VertexIndex: u32;
var<private> gl_Position: vec4<f32>;

fn main_1() {
    var tc: vec2<f32> = vec2<f32>(0.0, 0.0);
    var pos: vec2<f32>;

    let _e5: u32 = gl_VertexIndex;
    switch(_e5) {
        case 0u: {
            tc = vec2<f32>(1.0, 0.0);
        }
        case 1u: {
            tc = vec2<f32>(1.0, 1.0);
        }
        case 2u: {
            tc = vec2<f32>(0.0, 0.0);
        }
        case 3u: {
            tc = vec2<f32>(0.0, 1.0);
        }
        default: {
        }
    }
    let _e22: vec2<f32> = tc;
    v_TexCoord = _e22;
    let _e23: vec2<f32> = tc;
    pos = ((_e23 * 2.0) - vec2<f32>(1.0));
    let _e31: vec2<f32> = pos;
    let _e33: vec2<f32> = pos;
    gl_Position = vec4<f32>(_e31.x, -(_e33.y), 0.5, 1.0);
    return;
}

[[stage(vertex)]]
fn main([[builtin(vertex_index)]] param: u32) -> VertexOutput {
    gl_VertexIndex = param;
    main_1();
    let _e5: vec2<f32> = v_TexCoord;
    let _e7: vec4<f32> = gl_Position;
    return VertexOutput(_e5, _e7);
}
