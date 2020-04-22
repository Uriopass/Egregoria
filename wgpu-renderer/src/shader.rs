use std::{fs::File, io::prelude::*, path::PathBuf};

use wgpu::ShaderModule;

use crate::surface::Surface;

pub struct ShaderDescriptor {
    pub vert_shader: PathBuf,
    pub frag_shader: PathBuf,
}

impl ShaderDescriptor {
    // Todo: Make this function returning a struct instead of a tuple, not being able to know which element is which module
    // without checking the code isn't nice
    pub fn into_compiled_shaders(self, surface: &Surface) -> (ShaderModule, ShaderModule) {
        let vs_err_msg = format!(
            "Failed to open {} vertex shader file",
            self.vert_shader.to_str().unwrap()
        );
        let fs_err_msg = format!(
            "Failed to open {} fragment shader file",
            self.frag_shader.to_str().unwrap()
        );
        let mut vs_file = File::open(self.vert_shader).expect(vs_err_msg.as_str());
        let mut fs_file = File::open(self.frag_shader).expect(fs_err_msg.as_str());
        let mut vs_src = String::new();
        let mut fs_src = String::new();
        vs_file
            .read_to_string(&mut vs_src)
            .expect("Failed to read the content of {}");
        fs_file
            .read_to_string(&mut fs_src)
            .expect("Failed to read the content of {}");
        let vert_shader_spv =
            glsl_to_spirv::compile(&vs_src, glsl_to_spirv::ShaderType::Vertex).unwrap();
        let frag_shader_spv =
            glsl_to_spirv::compile(&fs_src, glsl_to_spirv::ShaderType::Fragment).unwrap();
        let vs_module = surface.create_shader_module(&vert_shader_spv);
        let fs_module = surface.create_shader_module(&frag_shader_spv);
        (vs_module, fs_module)
    }
}
