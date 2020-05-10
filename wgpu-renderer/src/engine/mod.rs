#[macro_use]
mod u8slice;

mod audio;
mod context;
mod draweables;
mod gfx;
mod input;
mod shader;
mod texture;
mod uniform;
mod vertex_types;

pub use audio::*;
pub use context::*;
pub use draweables::*;
pub use gfx::*;
pub use input::*;
pub use shader::*;
pub use texture::*;
pub use u8slice::*;
pub use uniform::*;
pub use vertex_types::*;

trait VBDesc {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a>;
}
