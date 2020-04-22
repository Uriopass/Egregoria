use wgpu::CommandBuffer;

pub trait Draweable: 'static {
    fn create_pipeline(gfx: &super::context::GfxContext) -> wgpu::RenderPipeline;
    fn draw(&self, gfx: &super::context::GfxContext) -> CommandBuffer;
}
