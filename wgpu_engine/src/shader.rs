use crate::wgpu::ShaderSource;
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use wgpu::{Device, ShaderModule};

pub struct CompiledShader(pub ShaderModule);

fn mk_module(data: String, device: &Device) -> ShaderModule {
    Device::create_shader_module(
        device,
        wgpu::ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Owned(data)),
        },
    )
}

/// if type isn't provided it will be detected by looking at extension
pub fn compile_shader(device: &Device, name: &str) -> CompiledShader {
    let mut p = PathBuf::new();
    p.push("assets/compiled_shaders");
    p.push(name.to_string() + ".wgsl");

    let mut source = std::fs::read_to_string(&p)
        .map_err(|e| {
            log::error!(
                "failed to read content of the shader {}: {}",
                p.to_string_lossy().into_owned(),
                e
            )
        })
        .unwrap();

    source = replace_imports(&p, source);

    let wgsl = mk_module(source, device);

    CompiledShader(wgsl)
}

fn replace_imports(base: &Path, src: String) -> String {
    src.lines()
        .map(|x| {
            if let Some(mut loc) = x.strip_prefix("#include \"") {
                println!("{}", x);
                loc = loc.strip_suffix('"').expect("include does not end with \"");
                let mut p = base.to_path_buf();
                p.pop();
                p.push(loc);
                return Cow::Owned(
                    std::fs::read_to_string(p).expect("could not find included file"),
                );
            }
            Cow::Borrowed(x)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
