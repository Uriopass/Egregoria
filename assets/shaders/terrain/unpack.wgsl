const MAX_HEIGHT:   f32 = 2008.0;
const MIN_HEIGHT:   f32 = -40.0;
const HEIGHT_RANGE: f32 = MAX_HEIGHT - MIN_HEIGHT;
const MAX_DIFF:     f32 = 32.0;

fn unpack_height(h: u32) -> f32 {
    return (f32(h) / 65535.0) * HEIGHT_RANGE + MIN_HEIGHT;
}

fn unpack_diffs(v: u32, lod_pow2: f32) -> vec2<f32> {
    let x = (f32(v >> 8u) - 128.0) / 127.0 * (MAX_DIFF * lod_pow2);
    let y = (f32(v & 0xFFu) - 128.0) / 127.0 * (MAX_DIFF * lod_pow2);
    return vec2<f32>(x, y);
}

fn unpack(v: u32, lod_pow2: f32) -> vec3<f32> {
    let h = unpack_height(v & 0xFFFFu);
    let d = unpack_diffs(v >> 16u, lod_pow2);
    return vec3(h, d);
}