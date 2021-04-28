struct RenderParams {
    mat4 invproj;
    vec4 ambiant;
    vec4 cam_pos;
    vec4 sun;
    vec2 viewport;
    float time;
    float ssao_strength;
    float ssao_radius;
    float ssao_falloff;
    float ssao_base;
    int ssao_samples;
    int ssao_enabled;
};