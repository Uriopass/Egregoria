use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct CompiledShader(pub Vec<u32>);

pub fn compile_shader(p: impl AsRef<Path>) -> CompiledShader {
    let p = p.as_ref();

    let mut extension = p.extension().unwrap().to_string_lossy().into_owned();
    extension.push_str(".spirv");
    let compiled_path = p.with_extension(extension);

    if let Ok(x) = File::open(compiled_path.clone()) {
        let data = wgpu::read_spirv(x).unwrap();
        return CompiledShader(data);
    }

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

    let mut spirv = glsl_to_spirv::compile(&src, shader_type).unwrap();

    std::io::copy(
        &mut spirv,
        &mut File::create(compiled_path).expect("Error opening the cached spirv file"),
    )
    .expect("Error trying to cache spirv output");

    let data = wgpu::read_spirv(&spirv).unwrap();
    CompiledShader(data)
}
