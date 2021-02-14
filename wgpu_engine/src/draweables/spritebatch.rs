use crate::{compile_shader, Drawable, GfxContext, Texture};
use geom::{LinearColor, Vec2};
use std::path::Path;
use std::rc::Rc;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, RenderPass, RenderPipeline, ShaderStage,
};

pub struct SpriteBatchBuilder {
    pub tex: Texture,
    instances: Vec<InstanceRaw>,
    stretch_x: f32,
    stretch_y: f32,
}

#[derive(Clone)]
pub struct SpriteBatch {
    instance_sbuffer: Rc<wgpu::Buffer>,
    instance_bg: Rc<BindGroup>,
    pub n_instances: u32,
    pub alpha_blend: bool,
    pub tex: Texture,
    pub tex_bg: Rc<BindGroup>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct InstanceRaw {
    tint: [f32; 4],
    pos: [f32; 3],
    _pad: f32,
    dir: [f32; 2],
    scale: [f32; 2],
}

u8slice_impl!(InstanceRaw);

impl SpriteBatchBuilder {
    pub fn from_path(ctx: &GfxContext, path: impl AsRef<Path>) -> Self {
        Self::new(Texture::from_path(ctx, path, None))
    }

    pub fn clear(&mut self) {
        self.instances.clear()
    }

    pub fn push(
        &mut self,
        pos: Vec2,
        direction: Vec2,
        z: f32,
        col: LinearColor,
        scale: (f32, f32),
    ) {
        self.instances.push(InstanceRaw {
            tint: col.into(),
            dir: direction.into(),
            scale: [scale.0 * self.stretch_x, -scale.1 * self.stretch_y],
            pos: [pos.x, pos.y, z],
            _pad: 0.0,
        })
    }

    pub fn new(tex: Texture) -> Self {
        let m = tex.extent.width.max(tex.extent.height) as f32;

        let stretch_x = tex.extent.width as f32 / m;
        let stretch_y = tex.extent.height as f32 / m;

        Self {
            stretch_x,
            stretch_y,
            tex,
            instances: vec![],
        }
    }

    pub fn build(&self, gfx: &GfxContext) -> Option<SpriteBatch> {
        let pipeline = gfx.get_pipeline::<SpriteBatch>();

        if self.instances.is_empty() {
            return None;
        }

        let instance_sbuffer = Rc::new(gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("spritebatch instance buffer"),
            contents: bytemuck::cast_slice(&self.instances),
            usage: wgpu::BufferUsage::STORAGE,
        }));

        let instance_bg = Rc::new(gfx.device.create_bind_group(&BindGroupDescriptor {
            label: Some("spritebatch instance bindgroup"),
            layout: &pipeline.get_bind_group_layout(2),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer {
                    buffer: &instance_sbuffer,
                    offset: 0,
                    size: None,
                },
            }],
        }));

        let tex_bg = Rc::new(
            self.tex
                .bindgroup(&gfx.device, &pipeline.get_bind_group_layout(0)),
        );

        Some(SpriteBatch {
            instance_sbuffer,
            instance_bg,
            n_instances: self.instances.len() as u32,
            alpha_blend: false,
            tex: self.tex.clone(),
            tex_bg,
        })
    }
}

impl Drawable for SpriteBatch {
    fn create_pipeline(gfx: &GfxContext) -> RenderPipeline {
        let vert = compile_shader("assets/shaders/spritebatch.vert", None);
        let frag = compile_shader("assets/shaders/spritebatch.frag", None);

        gfx.basic_pipeline(
            &[
                &Texture::bindgroup_layout(&gfx.device),
                &gfx.projection.layout,
                &gfx.device
                    .create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: Some("spritebatch instance bglayout"),
                        entries: &[BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStage::VERTEX,
                            ty: BindingType::Buffer {
                                has_dynamic_offset: false,
                                min_binding_size: None,
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                            },
                            count: None,
                        }],
                    }),
            ],
            &[],
            vert,
            frag,
        )
    }

    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let pipeline = &gfx.get_pipeline::<Self>();
        rp.set_pipeline(&pipeline);
        rp.set_bind_group(0, &self.tex_bg, &[]);
        rp.set_bind_group(1, &gfx.projection.bindgroup, &[]);
        rp.set_bind_group(2, &self.instance_bg, &[]);
        rp.draw(0..6 * self.n_instances, 0..1);
    }
}
