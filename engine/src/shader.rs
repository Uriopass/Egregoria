use crate::wgpu::ShaderSource;
use std::borrow::Cow;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Instant;
use wgpu::{Device, ShaderModule};

#[derive(Clone)]
pub struct CompiledModule(Rc<(ShaderModule, Vec<String>)>);

impl Deref for CompiledModule {
    type Target = ShaderModule;
    fn deref(&self) -> &Self::Target {
        &self.0 .0
    }
}

impl CompiledModule {
    pub fn get_deps(&self) -> &[String] {
        &self.0 .1
    }
}

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
pub fn compile_shader(device: &Device, name: &str) -> CompiledModule {
    let t = Instant::now();
    defer!(log::info!(
        "compiling shader {} took {:?}",
        name,
        t.elapsed()
    ));
    let mut p = PathBuf::new();
    p.push("assets/shaders");
    p.push(format!("{name}.wgsl"));

    let mut source = std::fs::read_to_string(&p)
        .map_err(|e| {
            log::error!(
                "failed to read content of the shader {}: {}",
                p.to_string_lossy().into_owned(),
                e
            )
        })
        .unwrap();

    let mut deps = vec![];
    source = replace_imports(&p, source, &mut deps);

    let wgsl = mk_module(source, device);

    CompiledModule(Rc::new((wgsl, deps)))
}

fn replace_imports(base: &Path, src: String, deps: &mut Vec<String>) -> String {
    src.lines()
        .map(move |x| {
            if let Some(mut loc) = x.strip_prefix("#include \"") {
                loc = loc.strip_suffix('"').expect("include does not end with \"");
                deps.push(loc.to_string());
                let mut p = base.to_path_buf();
                p.pop();
                p.push(loc);
                let mut s = std::fs::read_to_string(p)
                    .unwrap_or_else(|_| panic!("could not find included file {loc}"));
                s = replace_imports(base, s, deps);
                return Cow::Owned(s);
            }
            Cow::Borrowed(x)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
