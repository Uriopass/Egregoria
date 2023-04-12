fn tonemap(color: vec3<f32>) -> vec3<f32> {
    return 1.830796 * color / (color * 1.24068 + vec3(1.682186)); // approx of 1 - exp(-color) using scikit
    //return color / (1.0 + color);
    //return 1.0 - exp(-color);
}