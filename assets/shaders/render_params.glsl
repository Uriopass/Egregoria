struct RenderParams {
    mat4 invproj;
    mat4 sunproj;
    vec4 cam_pos;
    vec3 sun;
    vec4 sun_col;
    vec2 viewport;
    float time;
    float ssao_strength;
    float ssao_radius;
    float ssao_falloff;
    float ssao_base;
    int ssao_samples;
    int ssao_enabled;
    int shadow_mapping_enabled;
    int realistic_sky;
};