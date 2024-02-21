use crate::{compile_shader, CompiledModule, GfxContext};
use common::FastMap;
use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::Hash;
use std::path::Path;
use std::time::SystemTime;
use wgpu::{Device, ErrorFilter, RenderPipeline};

pub trait PipelineKey: Hash + 'static {
    fn build(
        &self,
        gfx: &GfxContext,
        mk_module: impl FnMut(&str, &[&str]) -> CompiledModule,
    ) -> RenderPipeline;
}

type ShaderPath = String;
type ShaderKey = (ShaderPath, Vec<String>);
type PipelineHash = u64;

#[derive(Default)]
pub struct Pipelines {
    pub(crate) shader_cache: BTreeMap<ShaderKey, CompiledModule>,
    pub(crate) shader_watcher: FastMap<ShaderPath, (Vec<ShaderPath>, Option<SystemTime>)>,
    pub(crate) pipelines:
        HashMap<PipelineHash, &'static RenderPipeline, common::TransparentHasherU64>,
    pub(crate) pipelines_deps: FastMap<ShaderPath, HashSet<PipelineHash>>,
}

impl Pipelines {
    pub fn new() -> Pipelines {
        Pipelines::default()
    }

    pub fn get_module(
        shader_cache: &mut BTreeMap<ShaderKey, CompiledModule>,
        shader_watcher: &mut FastMap<ShaderPath, (Vec<ShaderPath>, Option<SystemTime>)>,
        device: &Device,
        name: &str,
        defines: &FastMap<String, String>,
        extra_defines: Vec<String>,
    ) -> CompiledModule {
        let key = (name.to_string(), extra_defines);
        if let Some(v) = shader_cache.get(&key) {
            return v.clone();
        }
        shader_cache
            .entry(key)
            .or_insert_with_key(move |key| {
                let mut define_holder;
                let total_defines = {
                    if key.1.is_empty() {
                        defines
                    } else {
                        define_holder = defines.clone();
                        for k in &key.1 {
                            define_holder.insert(k.to_string(), "1".to_string());
                        }
                        &define_holder
                    }
                };

                let module = compile_shader(device, name, total_defines);

                for dep in module.get_deps() {
                    shader_watcher
                        .entry(dep.trim_end_matches(".wgsl").to_string())
                        .or_insert((vec![], None))
                        .0
                        .push(key.0.to_string());
                }
                shader_watcher
                    .entry(key.0.to_string())
                    .or_insert((vec![], None))
                    .0
                    .push(key.0.to_string());

                module
            })
            .clone()
    }

    pub fn get_pipeline(
        &mut self,
        gfx: &GfxContext,
        obj: impl PipelineKey,
        device: &Device,
    ) -> &'static RenderPipeline {
        let hash = common::hash_type_u64(&obj);
        match self.pipelines.entry(hash) {
            Entry::Occupied(o) => o.get(),
            Entry::Vacant(v) => {
                let mut deps = Vec::new();
                let pipeline = obj.build(gfx, |name, extra_defines| {
                    deps.push(name.to_string());

                    Pipelines::get_module(
                        &mut self.shader_cache,
                        &mut self.shader_watcher,
                        device,
                        name,
                        &gfx.defines,
                        extra_defines.iter().map(|x| x.to_string()).collect(),
                    )
                });
                for dep in deps {
                    self.pipelines_deps.entry(dep).or_default().insert(hash);
                }
                // ok to leak, we don't expect to build them many times in release
                v.insert(Box::leak(Box::new(pipeline)))
            }
        }
    }

    pub fn invalidate_all(&mut self) {
        self.pipelines.clear();
        self.shader_watcher.clear();
        self.shader_cache.clear();
        self.pipelines_deps.clear();
    }

    pub fn invalidate(
        &mut self,
        defines: &FastMap<String, String>,
        device: &Device,
        shader_name: &str,
    ) {
        for ((name, extra_defines), x) in self
            .shader_cache
            .range_mut((shader_name.to_string(), vec![])..)
        {
            if name != shader_name {
                break;
            }
            device.push_error_scope(ErrorFilter::Validation);
            let mut define_holder;
            let total_defines = {
                if extra_defines.is_empty() {
                    defines
                } else {
                    define_holder = defines.clone();
                    for k in extra_defines {
                        define_holder.insert(k.to_string(), "1".to_string());
                    }
                    &define_holder
                }
            };

            let new_shader = compile_shader(device, shader_name, total_defines);
            let scope = beul::execute(device.pop_error_scope());
            if scope.is_some() {
                log::error!("failed to compile shader for invalidation {}", shader_name);
                return;
            }
            *x = new_shader;
        }

        for hash in self
            .pipelines_deps
            .get_mut(shader_name)
            .unwrap_or(&mut HashSet::new())
            .drain()
        {
            self.pipelines.remove(&hash);
        }
    }

    pub fn check_shader_updates(&mut self, defines: &FastMap<String, String>, device: &Device) {
        let mut to_invalidate = HashSet::new();
        for (sname, (parents, entry)) in &mut self.shader_watcher {
            let meta = unwrap_cont!(std::fs::metadata(Path::new(&format!(
                "assets/shaders/{sname}.wgsl"
            )))
            .ok());
            let filetime = unwrap_cont!(meta.modified().ok());
            match entry.as_mut() {
                Some(entry) => {
                    if *entry < filetime {
                        to_invalidate.insert(sname.clone());
                        to_invalidate.extend(parents.iter().cloned());
                        *entry = filetime;
                    }
                }
                None => {
                    *entry = Some(filetime);
                }
            }
        }
        for sname in to_invalidate {
            log::info!("invalidating shader {}", sname);
            self.invalidate(defines, device, &sname);
        }
    }
}
