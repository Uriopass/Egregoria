// 16-aligned!
struct LightUpdate {
    data: vec4<u32>,
    data2: vec4<u32>,
    position: vec2<u32>,
}

@group(0) @binding(0) var light_tex:   texture_storage_2d<rgba32uint, write>;
@group(0) @binding(1) var light_tex_2: texture_storage_2d<rgba32uint, write>;

@group(1) @binding(0) var<storage, read> light_changes: array<LightUpdate>;

@compute @workgroup_size(64,1,1)
fn main(
  @builtin(global_invocation_id) id: vec3<u32>,
) {
  let i: u32 = id.x;
  if (i >= arrayLength(&light_changes)) {
    return;
  }
  let position = light_changes[i].position;

  textureStore(light_tex,   position, light_changes[i].data);
  textureStore(light_tex_2, position, light_changes[i].data2);
}