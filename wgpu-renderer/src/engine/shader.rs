use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct CompiledShader(pub Vec<u32>);

pub fn compile_shader(p: impl AsRef<Path>) -> CompiledShader {
    let p = p.as_ref();
    let mut file = File::open(p).unwrap_or_else(|_| panic!("Failed to open {:?} shader file", p));
    let mut src = String::new();
    file.read_to_string(&mut src)
        .expect("Failed to read the content of the shader");

    let extension = p.extension().unwrap();

    let shader_type = if extension == "frag" {
        glsl_to_spirv::ShaderType::Fragment
    } else if extension == "vert" {
        glsl_to_spirv::ShaderType::Vertex
    } else {
        panic!(
            "Unexpected shader extension: {}",
            &extension.to_string_lossy()
        );
    };

    let spirv = glsl_to_spirv::compile(&src, shader_type).unwrap();
    let data = wgpu::read_spirv(&spirv).unwrap();
    CompiledShader(data)
}
