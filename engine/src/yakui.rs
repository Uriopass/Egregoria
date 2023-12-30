use crate::{GfxContext, GuiRenderContext};
use wgpu::TextureFormat;
use winit::window::Window;
use yakui::font::{Font, FontSettings, Fonts};
use yakui::Yakui;

pub struct YakuiWrapper {
    pub yakui: Yakui,
    pub renderer: yakui_wgpu::YakuiWgpu,
    platform: yakui_winit::YakuiWinit,
    pub zoom_factor: f32,
    pub format: TextureFormat,
}

impl YakuiWrapper {
    pub fn new(gfx: &GfxContext, el: &Window) -> Self {
        let yakui = Yakui::new();

        let fonts = yakui.dom().get_global_or_init(Fonts::default);
        let font = Font::from_bytes(
            include_bytes!("../../assets/font_awesome_solid_900.otf").as_slice(),
            FontSettings::default(),
        )
        .unwrap();

        fonts.add(font, Some("icons"));

        let platform = yakui_winit::YakuiWinit::new(el);

        let renderer = yakui_wgpu::YakuiWgpu::new(&gfx.device, &gfx.queue);

        Self {
            yakui,
            renderer,
            platform,
            zoom_factor: 1.0,
            format: gfx.fbos.format,
        }
    }

    pub fn render(&mut self, gfx: &mut GuiRenderContext<'_>, ui_render: impl for<'ui> FnOnce()) {
        self.yakui.set_scale_factor(self.zoom_factor);

        self.yakui.start();
        ui_render();
        self.yakui.finish();

        self.renderer.paint_with_encoder(
            &mut self.yakui,
            &gfx.gfx.device,
            &gfx.gfx.queue,
            gfx.encoder,
            yakui_wgpu::SurfaceInfo {
                format: self.format,
                sample_count: gfx.gfx.samples,
                color_attachment: &gfx.gfx.fbos.color_msaa,
                resolve_target: Some(gfx.view),
            },
        );
    }

    pub fn handle_event(&mut self, e: &winit::event::Event<()>) -> bool {
        self.platform.handle_event(&mut self.yakui, e)
    }
}
