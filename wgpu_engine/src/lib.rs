#[macro_use]
extern crate common;

#[macro_use]
pub mod u8slice;

mod draweables;
mod geometry;
mod gfx;
pub mod lighting;
mod shader;
mod texture;
mod uniform;
mod vertex_types;

pub use draweables::*;
pub use geometry::*;
pub use gfx::*;
pub use shader::*;
pub use texture::*;
pub use u8slice::*;
pub use uniform::*;
pub use vertex_types::*;

pub use wgpu;

trait VBDesc {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}
