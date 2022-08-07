use crate::wgpu::ShaderSource;
use std::borrow::Cow;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use wgpu::ShaderModule;

#[derive(Copy, Clone)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

pub struct CompiledShader(pub ShaderModule, pub ShaderType);

pub enum CacheState {
    Nofile,
    Outdated(CompiledShader),
    Fresh(CompiledShader),
}

fn cache_filename(p: &Path) -> Option<PathBuf> {
    let mut name = p.file_name()?.to_string_lossy().into_owned();
    name.push_str(".wgsl");

    Some(p.parent()?.parent()?.join("compiled_shaders").join(name))
}

fn mk_module(data: &str, device: &wgpu::Device) -> ShaderModule {
    wgpu::Device::create_shader_module(
        device,
        wgpu::ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(data)),
        },
    )
}

fn find_in_cache(
    device: &wgpu::Device,
    compiled_path: &Path,
    stype: ShaderType,
    last_modified: SystemTime,
) -> CacheState {
    let read = match std::fs::read_to_string(&compiled_path) {
        Ok(x) => x,
        Err(_) => return CacheState::Nofile,
    };

    let shader = CompiledShader(mk_module(&*read, device), stype);

    let f = match File::open(compiled_path) {
        Ok(x) => x,
        Err(_) => return CacheState::Nofile,
    };

    let cached_last_modified = match f.metadata() {
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

fn save_to_cache(compiled_path: &Path, wgsl: &str) -> Option<()> {
    std::fs::create_dir_all(compiled_path.parent()?).ok()?;
    std::fs::write(compiled_path, wgsl).ok()?;
    Some(())
}

/// if type isn't provided it will be detected by looking at extension
pub fn compile_shader(
    device: &wgpu::Device,
    p: impl AsRef<Path>,
    stype: Option<ShaderType>,
) -> CompiledShader {
    let p = p.as_ref();

    let compiled_name = cache_filename(p);

    let stype = match stype {
        Some(x) => x,
        None => {
            let extension = p.extension().expect("invalid shader extension");

            match extension.to_string_lossy().into_owned().as_ref() {
                "frag" | "glslf" => ShaderType::Fragment,
                "vert" | "glslv" => ShaderType::Vertex,
                _ => panic!(
                    "Unexpected shader extension: {}",
                    &extension.to_string_lossy()
                ),
            }
        }
    };

    let mut sfile = File::open(p).unwrap_or_else(|_| {
        panic!(
            "Failed to open {:?}. Did you run the binary next to the assets ?",
            p
        )
    });

    let cache_state =
        if let Some(last_modified) = sfile.metadata().ok().and_then(|x| x.modified().ok()) {
            if let Some(x) = &compiled_name {
                find_in_cache(device, x, stype, last_modified)
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
            log::warn!(
                r#"Shader "{}" was found in cache, but is outdated, recompiling if possible"#,
                p.to_string_lossy().into_owned()
            );
            Some(x)
        }
        CacheState::Nofile => {
            log::warn!(
                r#"Shader "{}" not found in cache, recompiling"#,
                p.to_string_lossy().into_owned()
            );
            None
        }
    };

    let mut src = String::new();
    let fileread = sfile.read_to_string(&mut src).map_err(|e| {
        log::warn!(
            "failed to read content of the shader {}: {}",
            p.to_string_lossy().into_owned(),
            e
        )
    });

    let wgsl = match fileread.ok().and_then(|_| compile(p, src, stype)) {
        Some(x) => {
            log::info!("successfully compiled {}", p.to_string_lossy().into_owned());
            x
        }
        None => {
            return outdated
                .map(|x| {
                    log::warn!(
                        "couldn't compile {}: using outdated shader",
                        p.to_string_lossy().into_owned()
                    );
                    x
                })
                .expect("couldn't compile glsl and no outdated wgsl found in cache, aborting.");
        }
    };

    let _ = compiled_name.and_then(|x| save_to_cache(&x, &wgsl));

    CompiledShader(mk_module(&*wgsl, device), stype)
}

fn compile(p: &Path, mut src: String, stype: ShaderType) -> Option<String> {
    log::info!("Using naga compiler");
    src = src
        .lines()
        .map(|x| {
            if let Some(mut loc) = x.strip_prefix("#include \"") {
                loc = loc.strip_suffix('"').expect("include does not end with \"");
                let mut p = p.to_path_buf();
                p.pop();
                p.push(loc);
                return Cow::Owned(
                    std::fs::read_to_string(p).expect("could not find included file"),
                );
            }
            Cow::Borrowed(x)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let glsl = naga::front::glsl::Parser::default()
        .parse(
            &naga::front::glsl::Options {
                stage: match stype {
                    ShaderType::Vertex => naga::ShaderStage::Vertex,
                    ShaderType::Fragment => naga::ShaderStage::Fragment,
                },
                defines: Default::default(),
            },
            &src,
        )
        .map_err(|e| log::error!("{:?}", e))
        .ok()?;

    let mut valid = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    );
    let info = valid
        .validate(&glsl)
        .map_err(|e| log::error!("{:?}", e))
        .ok()?;

    let mut wgsl = String::new();
    naga::back::wgsl::Writer::new(&mut wgsl, naga::back::wgsl::WriterFlags::all())
        .write(&glsl, &info)
        .ok()?;

    Some(wgsl)
}
