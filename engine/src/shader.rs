#![allow(clippy::redundant_pattern_matching)]

use crate::wgpu::ShaderSource;
use common::FastMap;
use std::borrow::Cow;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use wgpu::{Device, ShaderModule};

#[derive(Clone)]
pub struct CompiledModule(Arc<(ShaderModule, Vec<String>)>);

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
/// name shouldn't include "assets/shaders" or ".wgsl" extension. It will be added automatically
pub fn compile_shader(
    device: &Device,
    name: &str,
    defines: &FastMap<String, String>,
) -> CompiledModule {
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
    source = apply_ifdefs_and_imports(&p, defines, &source, &mut deps);

    let wgsl = mk_module(source, device);

    CompiledModule(Arc::new((wgsl, deps)))
}

/// apply_ifdefs updates the source taking into account #ifdef and #ifndef
/// syntax is as follow:
/// #ifdef <name> or #ifndef <name>
///   <code>
/// #else or #elif <name>
///  <code>
/// #endif
///
/// the ifdefs can be nested
fn apply_ifdefs_and_imports(
    base: &Path,
    defines: &FastMap<String, String>,
    src: &str,
    deps: &mut Vec<String>,
) -> String {
    // A stack of:
    // whether that nest level is true
    // whether we've seen a true yet in the if/elif chain
    let mut ifdef_stack: Vec<(bool, bool)> = vec![];

    src.lines()
        .map(|line| {
            let x = line.trim();
            if let Some(mut ifdef) = x.strip_prefix("#ifdef ") {
                ifdef = ifdef.trim();
                let should_execute = defines.contains_key(ifdef);
                ifdef_stack.push((should_execute, should_execute));
                return Cow::Borrowed("");
            }
            if let Some(mut ifndef) = x.strip_prefix("#ifndef ") {
                ifndef = ifndef.trim();
                let should_execute = !defines.contains_key(ifndef);
                ifdef_stack.push((should_execute, should_execute));
                return Cow::Borrowed("");
            }
            if let Some(_) = x.strip_prefix("#else") {
                let (val, has_true) = ifdef_stack.last_mut().unwrap();
                *val = !*val && !*has_true;
                return Cow::Borrowed("");
            }
            if let Some(mut elif) = x.strip_prefix("#elifdef ") {
                elif = elif.trim();
                let (val, has_true) = ifdef_stack.last_mut().unwrap();
                *val = !*val && defines.contains_key(elif);
                *has_true = *has_true || *val;
                return Cow::Borrowed("");
            }
            if let Some(_) = x.strip_prefix("#endif") {
                ifdef_stack.pop();
                return Cow::Borrowed("");
            }
            if ifdef_stack.iter().any(|(val, _)| !*val) {
                return Cow::Borrowed("");
            }
            if let Some(mut loc) = x.strip_prefix("#include \"") {
                loc = loc.strip_suffix('"').expect("include does not end with \"");
                deps.push(loc.to_string());
                let mut p = base.to_path_buf();
                p.pop();
                p.push(loc);
                let mut s = std::fs::read_to_string(&p)
                    .unwrap_or_else(|_| panic!("could not find included file {loc} for {base:?}"));
                s = apply_ifdefs_and_imports(&p, defines, &s, deps);
                return Cow::Owned(s);
            }
            Cow::Borrowed(line)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
