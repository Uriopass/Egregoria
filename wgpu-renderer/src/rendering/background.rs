use crate::engine::{
    compile_shader, CompiledShader, FrameContext, GfxContext, ShadedQuad, Shaders,
};

struct Background;

impl Shaders for Background {
    fn vert_shader() -> CompiledShader {
        compile_shader("resources/shaders/background.vert", None)
    }

    fn frag_shader() -> CompiledShader {
        compile_shader("resources/shaders/background.frag", None)
    }
}

pub fn prepare_background(gfx: &mut GfxContext) {
    gfx.register_pipeline::<ShadedQuad<Background>>();
}

pub fn draw_background(fctx: &mut FrameContext) {
    let sq = ShadedQuad::<Background>::new(fctx.gfx);

    fctx.objs.push(Box::new(sq));
}
