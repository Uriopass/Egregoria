#[macro_use]
mod u8slice;

mod audio;
mod colored_uv_vertex;
mod context;
mod draweables;
mod gfx;
mod input;
mod shader;
mod texture;
mod uniform;
mod uv_vertex;
mod vertex;

pub use audio::*;
pub use colored_uv_vertex::*;
pub use context::*;
pub use draweables::*;
pub use gfx::*;
pub use input::*;
pub use shader::*;
pub use texture::*;
pub use u8slice::*;
pub use uniform::*;
pub use uv_vertex::*;
pub use vertex::*;

trait VBDesc {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a>;
}
