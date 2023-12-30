#![allow(clippy::redundant_pattern_matching)]

use crate::wgpu::ShaderSource;
use common::FastMap;
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
    source = replace_imports(&p, &source, &mut deps);
    source = apply_ifdefs(defines, &source);

    let wgsl = mk_module(source, device);

    CompiledModule(Rc::new((wgsl, deps)))
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
fn apply_ifdefs(defines: &FastMap<String, String>, src: &str) -> String {
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
                return "";
            }
            if let Some(mut ifndef) = x.strip_prefix("#ifndef ") {
                ifndef = ifndef.trim();
                let should_execute = !defines.contains_key(ifndef);
                ifdef_stack.push((should_execute, should_execute));
                return "";
            }
            if let Some(_) = x.strip_prefix("#else") {
                let (val, has_true) = ifdef_stack.last_mut().unwrap();
                *val = !*val && !*has_true;
                return "";
            }
            if let Some(mut elif) = x.strip_prefix("#elifdef ") {
                elif = elif.trim();
                let (val, has_true) = ifdef_stack.last_mut().unwrap();
                *val = !*val && defines.contains_key(elif);
                *has_true = *has_true || *val;
                return "";
            }
            if let Some(_) = x.strip_prefix("#endif") {
                ifdef_stack.pop();
                return "";
            }
            if ifdef_stack.iter().any(|(val, _)| !*val) {
                return "";
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn replace_imports(base: &Path, src: &str, deps: &mut Vec<String>) -> String {
    src.lines()
        .map(move |x| {
            if let Some(mut loc) = x.strip_prefix("#include \"") {
                loc = loc.strip_suffix('"').expect("include does not end with \"");
                deps.push(loc.to_string());
                let mut p = base.to_path_buf();
                p.pop();
                p.push(loc);
                let mut s = std::fs::read_to_string(&p)
                    .unwrap_or_else(|_| panic!("could not find included file {loc} for {base:?}"));
                s = replace_imports(&p, &s, deps);
                return Cow::Owned(s);
            }
            Cow::Borrowed(x)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use common::FastMap;

    #[test]
    fn test_apply_ifdefs() {
        let src = r#"
        #ifdef A
        a
        #else
        b
        #endif
        "#;

        fn f(x: &[&str]) -> FastMap<String, String> {
            x.iter().map(|x| (x.to_string(), "".to_string())).collect()
        }

        assert_eq!(super::apply_ifdefs(&f(&[]), src).trim(), "b");
        assert_eq!(super::apply_ifdefs(&f(&["A"]), src).trim(), "a");
        assert_eq!(super::apply_ifdefs(&f(&["B"]), src).trim(), "b");
        assert_eq!(super::apply_ifdefs(&f(&["A", "B"]), src).trim(), "a");

        let src = r#"
        #ifdef A
        a
        #elifdef B
        b
        #else
        c
        #endif
        "#;

        assert_eq!(super::apply_ifdefs(&f(&[]), src).trim(), "c");
        assert_eq!(super::apply_ifdefs(&f(&["A"]), src).trim(), "a");
        assert_eq!(super::apply_ifdefs(&f(&["B"]), src).trim(), "b");
        assert_eq!(super::apply_ifdefs(&f(&["A", "B"]), src).trim(), "a");

        let src = r#"
        #ifdef A
        #ifdef B
        a
        #else
        b
        #endif
        #else
        c
        #endif
        "#;

        assert_eq!(super::apply_ifdefs(&f(&[]), src).trim(), "c");
        assert_eq!(super::apply_ifdefs(&f(&["A"]), src).trim(), "b");
        assert_eq!(super::apply_ifdefs(&f(&["B"]), src).trim(), "c");
        assert_eq!(super::apply_ifdefs(&f(&["A", "B"]), src).trim(), "a");
    }
}
