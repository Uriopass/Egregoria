use imgui_wgpu::{Renderer, RendererConfig};
use std::time::Instant;
use wgpu_engine::{GfxContext, GuiRenderContext};
use winit::window::Window;

pub struct ImguiWrapper {
    pub imgui: imgui::Context,
    renderer: imgui_wgpu::Renderer,
    last_frame: Instant,
    platform: imgui_winit_support::WinitPlatform,
    pub last_mouse_captured: bool,
    pub last_kb_captured: bool,
}

impl ImguiWrapper {
    pub fn new(gfx: &mut GfxContext, window: &Window) -> Self {
        let mut imgui = imgui::Context::create();

        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );

        let font_size = 13.0 as f32;
        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            }]);

        let renderer = RendererConfig::new()
            .set_texture_format(gfx.sc_desc.format)
            .build(&mut imgui, &gfx.device, &gfx.queue);
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
        mut gfx: GuiRenderContext,
        window: &Window,
        goria: &mut egregoria::Egregoria,
        gui: &mut gui::Gui,
    ) {
        let now = Instant::now();
        let delta = now - self.last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame = now;

        self.imgui.io_mut().delta_time = delta_s;

        // Prepare
        self.platform
            .prepare_frame(self.imgui.io_mut(), window)
            .expect("Failed to prepare frame");

        let ui: imgui::Ui = self.imgui.frame();
        gui.render(&ui, goria);

        self.last_mouse_captured = ui.io().want_capture_mouse;
        self.last_kb_captured = ui.io().want_capture_keyboard;

        self.platform.prepare_render(&ui, window);

        let mut rpass = gfx.rpass.take().unwrap();
        let _ = self
            .renderer
            .render(ui.render(), gfx.queue, gfx.device, &mut rpass)
            .map_err(|err| log::error!("Error rendering the UI: {:?}", err));
    }

    pub fn handle_event(&mut self, window: &Window, e: &winit::event::Event<()>) {
        self.platform.handle_event(self.imgui.io_mut(), window, e);
    }
}
