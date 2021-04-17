// A single iteration of Bob Jenkins' One-At-A-Time hashing algorithm.
fn hash(mut x: u32) -> u32 {
    x = x.wrapping_add(x << 10u32);
    x ^= x >> 6u32;
    x = x.wrapping_add(x << 3u32);
    x ^= x >> 11u32;
    x = x.wrapping_add(x << 15u32);
    x
}

// Compound versions of the hashing algorithm I whipped together.
fn hash2(x: u32, y: u32) -> u32 {
    hash(x ^ hash(y))
}
fn hash3(x: u32, y: u32, z: u32) -> u32 {
    hash(x ^ hash(y) ^ hash(z))
}

// Construct a float with half-open range [0:1] using low 23 bits.
// All zeroes yields 0.0, all ones yields the next smallest representable value below 1.0.
fn float_construct(mut m: u32) -> f32 {
    const IEEE_MANTISSA: u32 = 0x007FFFFFu32; // binary32 mantissa bitmask
    const IEEE_ONE: u32 = 0x3F800000u32; // 1.0 in IEEE binary32

    m &= IEEE_MANTISSA; // Keep only mantissa bits (fractional part)
    m |= IEEE_ONE; // Add fractional part to 1.0

    let f: f32 = f32::from_bits(m); // Range [1:2]
    f - 1.0 // Range [0:1]
}

// Pseudo-random value in half-open range [0:1].
pub fn rand(x: f32) -> f32 {
    float_construct(hash(x.to_bits()))
}
pub fn rand2(x: f32, y: f32) -> f32 {
    float_construct(hash2(x.to_bits(), y.to_bits()))
}
pub fn rand3(x: f32, y: f32, z: f32) -> f32 {
    float_construct(hash3(x.to_bits(), y.to_bits(), z.to_bits()))
}

pub fn randu(x: u32) -> f32 {
    float_construct(hash(x))
}
