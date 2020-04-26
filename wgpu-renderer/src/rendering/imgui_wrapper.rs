use crate::engine::{FrameContext, GfxContext};
use imgui_wgpu::Renderer;
use std::time::Instant;

pub struct ImguiWrapper {
    imgui: scale::imgui::Context,
    renderer: imgui_wgpu::Renderer,
    last_frame: Instant,
    platform: imgui_winit_support::WinitPlatform,
    pub last_mouse_captured: bool,
    pub last_kb_captured: bool,
}

impl ImguiWrapper {
    pub fn new(gfx: &mut GfxContext) -> Self {
        let mut imgui = scale::imgui::Context::create();

        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            &gfx.window,
            imgui_winit_support::HiDpiMode::Default,
        );

        let font_size = 13.0 as f32;
        imgui
            .fonts()
            .add_font(&[scale::imgui::FontSource::DefaultFontData {
                config: Some(scale::imgui::FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            }]);

        let renderer = Renderer::new(
            &mut imgui,
            &gfx.device,
            &mut gfx.queue,
            gfx.sc_desc.format,
            None,
        );
        Self {
            imgui,
            renderer,
            last_frame: Instant::now(),
            last_mouse_captured: false,
            last_kb_captured: false,
            platform,
        }
    }

    pub fn render(
        &mut self,
        gfx: &mut FrameContext,
        world: &mut scale::specs::World,
        gui: &mut scale::gui::Gui,
    ) {
        let now = Instant::now();
        let delta = now - self.last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame = now;

        self.imgui.io_mut().delta_time = delta_s;

        // Prepare
        self.platform
            .prepare_frame(self.imgui.io_mut(), &gfx.gfx.window)
            .expect("Failed to prepare frame");

        let ui: scale::imgui::Ui = self.imgui.frame();
        gui.render(&ui, world);
        self.last_mouse_captured = ui.io().want_capture_mouse;
        self.last_kb_captured = ui.io().want_capture_keyboard;

        self.platform.prepare_render(&ui, &gfx.gfx.window);

        self.renderer
            .render(
                ui.render(),
                &gfx.gfx.device,
                &mut gfx.encoder,
                &gfx.frame.view,
            )
            .unwrap();
    }

    pub fn handle_event(&mut self, gfx: &GfxContext, e: &winit::event::Event<()>) {
        self.platform
            .handle_event(self.imgui.io_mut(), &gfx.window, e);
    }
}
