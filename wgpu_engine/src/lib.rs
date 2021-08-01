#![deny(
    rustdoc::all,
    rust_2018_compatibility,
    rust_2018_idioms,
    nonstandard_style,
    unused,
    future_incompatible,
    unused_extern_crates,
    clippy::all,
    clippy::doc_markdown,
    clippy::wildcard_imports
)]
#![allow(
    clippy::collapsible_else_if,
    clippy::manual_range_contains,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix,
    clippy::blocks_in_if_conditions,
    clippy::upper_case_acronyms,
    clippy::must_use_candidate,
    missing_copy_implementations,
    missing_debug_implementations
)]

#[macro_use]
extern crate common;

#[macro_use]
pub mod u8slice;

mod drawables;
mod geometry;
mod gfx;
pub mod lighting;
pub mod meshload;
pub mod pbuffer;
mod shader;
mod texture;
mod uniform;
mod vertex_types;

pub use drawables::*;
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
