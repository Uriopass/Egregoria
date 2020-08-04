use glsl_to_spirv::{ShaderType, SpirvOutput};
use std::fs::File;
use std::io::Read;
use std::panic::catch_unwind;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub struct CompiledShader(pub Vec<u32>, pub ShaderType);

pub enum CacheState {
    Nofile,
    InvalidSpirv,
    Outdated(CompiledShader),
    Fresh(CompiledShader),
}

fn cache_filename(p: &Path) -> Option<PathBuf> {
    let mut name = p.file_name()?.to_string_lossy().into_owned();
    name.push_str(".spirv");

    Some(p.parent()?.parent()?.join("compiled_shaders").join(name))
}

fn find_in_cache(
    compiled_path: &PathBuf,
    stype: ShaderType,
    last_modified: SystemTime,
) -> CacheState {
    let x = match File::open(&compiled_path) {
        Ok(x) => x,
        Err(_) => return CacheState::Nofile,
    };

    let data = match wgpu::read_spirv(&x) {
        Ok(x) => x,
        Err(_) => return CacheState::InvalidSpirv,
    };

    let shader = CompiledShader(data, stype);

    let cached_last_modified = match x.metadata() {
        Ok(x) => match x.modified() {
            Ok(x) => x,
            Err(_) => return CacheState::Outdated(shader),
        },
        Err(_) => return CacheState::Outdated(shader),
    };

    if last_modified
        .duration_since(cached_last_modified)
        .map_or(true, |d| d.as_secs_f32() < 10.0)
    {
        CacheState::Fresh(shader)
    } else {
        CacheState::Outdated(shader)
    }
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
            let extension = p.extension().expect("invalid shader extension");

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

    let mut sfile = File::open(p).unwrap_or_else(|_| panic!("Failed to open {:?} shader file", p));

    let cache_state =
        if let Some(last_modified) = sfile.metadata().ok().and_then(|x| x.modified().ok()) {
            if let Some(x) = &compiled_name {
                find_in_cache(x, stype.clone(), last_modified)
            } else {
                CacheState::Nofile
            }
        } else {
            CacheState::Nofile
        };

    let outdated: Option<CompiledShader> = match cache_state {
        CacheState::Fresh(x) => {
            return x;
        }
        CacheState::Outdated(x) => {
            println!(
                r#"Shader "{}" was found in cache, but is outdated, recompiling if possible"#,
                p.to_string_lossy().into_owned()
            );
            Some(x)
        }
        CacheState::Nofile => {
            println!(
                r#"Shader "{}" not found in cache, recompiling"#,
                p.to_string_lossy().into_owned()
            );
            None
        }
        CacheState::InvalidSpirv => {
            println!(
                r#"Shader "{}" was found in cache but is invalid, recompiling"#,
                p.to_string_lossy().into_owned()
            );
            None
        }
    };

    let mut src = String::new();
    sfile
        .read_to_string(&mut src)
        .expect("Failed to read the content of the shader");

    let mut spirv = match catch_unwind(|| glsl_to_spirv::compile(&src, stype.clone()).unwrap()) {
        Ok(x) => x,
        Err(_) => {
            return outdated
                .expect("Couldn't compile glsl and no outdated spirv found in cache, aborting.");
        }
    };

    let _ = compiled_name.and_then(|x| save_to_cache(&x, &mut spirv));

    let data = wgpu::read_spirv(&spirv).expect("Error trying to decode spirv");
    CompiledShader(data, stype)
}
