use glsl_to_spirv::SpirvOutput;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct CompiledShader(pub Vec<u32>);

fn cache_filename(p: &Path) -> Option<PathBuf> {
    let mut name = p.file_name()?.to_string_lossy().into_owned();
    name.push_str(".spirv");

    Some(p.parent()?.parent()?.join("compiled_shaders").join(name))
}

fn find_in_cache(compiled_path: &PathBuf) -> Option<CompiledShader> {
    let x = File::open(&compiled_path).ok()?;
    let data = wgpu::read_spirv(x).ok()?;

    Some(CompiledShader(data))
}

fn save_to_cache(compiled_path: &PathBuf, spirv: &mut SpirvOutput) -> Option<()> {
    std::fs::create_dir_all(compiled_path.parent()?).ok()?;
    std::io::copy(spirv, &mut File::create(compiled_path).ok()?).ok()?;
    Some(())
}

pub fn compile_shader(p: impl AsRef<Path>) -> CompiledShader {
    let p = p.as_ref();

    let compiled_name = cache_filename(p);

    if let Some(x) = compiled_name.as_ref().and_then(|x| find_in_cache(&x)) {
        return x;
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

    compiled_name.and_then(|x| save_to_cache(&x, &mut spirv));

    let data = wgpu::read_spirv(&spirv).unwrap();
    CompiledShader(data)
}
