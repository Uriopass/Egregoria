use geom::{LinearColor, Vec3};
use wgpu_engine::{
    compile_shader, CompiledShader, FrameContext, GfxContext, ShadedQuad, Shaders, Uniform,
};

struct Background;

impl Shaders for Background {
    fn vert_shader() -> CompiledShader {
        compile_shader("assets/shaders/background.vert", None)
    }

    fn frag_shader() -> CompiledShader {
        compile_shader("assets/shaders/background.frag", None)
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
struct BackgroundUniform {
    sea_color: LinearColor,
    grass_color: LinearColor,
    sand_color: LinearColor,
    time: f32,
    _pad: Vec3,
}

wgpu_engine::u8slice_impl!(BackgroundUniform);

pub fn prepare_background(gfx: &mut GfxContext) {
    gfx.register_pipeline::<ShadedQuad<Background, BackgroundUniform>>();
}

pub fn draw_background(fctx: &mut FrameContext) {
    let sq = ShadedQuad::<Background, BackgroundUniform>::new(
        fctx.gfx,
        Uniform::new(
            BackgroundUniform {
                sea_color: common::config().water_col.into(),
                grass_color: common::config().ground_base_col.into(),
                sand_color: common::config().sand_col.into(),
                time: fctx.gfx.time_uni.value,
                _pad: Vec3::ZERO,
            },
            &fctx.gfx.device,
        ),
    );

    fctx.objs.push(Box::new(sq));
}
