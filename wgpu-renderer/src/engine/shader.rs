use glsl_to_spirv::{ShaderType, SpirvOutput};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct CompiledShader(pub Vec<u32>, pub ShaderType);

fn cache_filename(p: &Path) -> Option<PathBuf> {
    let mut name = p.file_name()?.to_string_lossy().into_owned();
    name.push_str(".spirv");

    Some(p.parent()?.parent()?.join("compiled_shaders").join(name))
}

fn find_in_cache(compiled_path: &PathBuf, stype: ShaderType) -> Option<CompiledShader> {
    let x = File::open(&compiled_path).ok()?;
    let data = wgpu::read_spirv(x).ok()?;

    Some(CompiledShader(data, stype))
}

fn save_to_cache(compiled_path: &PathBuf, spirv: &mut SpirvOutput) -> Option<()> {
    std::fs::create_dir_all(compiled_path.parent()?).ok()?;
    std::io::copy(spirv, &mut File::create(compiled_path).ok()?).ok()?;
    Some(())
}

/// if type isn't provided it will be detected by looking at extension
pub fn compile_shader(p: impl AsRef<Path>, stype: Option<ShaderType>) -> CompiledShader {
    let p = p.as_ref();

    let compiled_name = cache_filename(p);

    let stype = match stype {
        Some(x) => x,
        None => {
            let extension = p.extension().unwrap();

            match extension.to_string_lossy().into_owned().as_ref() {
                "frag" | "glslf" => glsl_to_spirv::ShaderType::Fragment,
                "vert" | "glslv" => glsl_to_spirv::ShaderType::Vertex,
                _ => panic!(
                    "Unexpected shader extension: {}",
                    &extension.to_string_lossy()
                ),
            }
        }
    };

    if let Some(x) = compiled_name
        .as_ref()
        .and_then(|x| find_in_cache(&x, stype.clone()))
    {
        return x;
    }

    let mut file = File::open(p).unwrap_or_else(|_| panic!("Failed to open {:?} shader file", p));
    let mut src = String::new();
    file.read_to_string(&mut src)
        .expect("Failed to read the content of the shader");

    let mut spirv = glsl_to_spirv::compile(&src, stype.clone()).unwrap();

    compiled_name.and_then(|x| save_to_cache(&x, &mut spirv));

    let data = wgpu::read_spirv(&spirv).unwrap();
    CompiledShader(data, stype)
}
