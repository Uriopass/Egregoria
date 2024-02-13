const HEIGHT_RANGE: f32 = 2048.0;
const MIN_HEIGHT: f32 = -40.0007;
const MAX_HEIGHT: f32 = HEIGHT_RANGE - MIN_HEIGHT;

fn unpack_height(h: u32) -> f32 {
    return (f32(h) / 65535.0) * HEIGHT_RANGE + MIN_HEIGHT;
}

fn unpack_normal(v: u32) -> vec3<f32> {
    let x = f32(v >> 8u) / 128.0 - 1.0;
    let y = f32(v & 0xFFu) / 128.0 - 1.0;
    let z = sqrt(max(0.0, 1.0 - x * x - y * y));

    return vec3<f32>(x, y, z);
}
