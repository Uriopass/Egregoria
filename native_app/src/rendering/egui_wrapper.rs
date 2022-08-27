use egui::{FontData, FontDefinitions};
use egui_wgpu::renderer;
use egui_wgpu::renderer::ScreenDescriptor;
use egui_wgpu::wgpu::TextureFormat;
use std::time::Instant;
use wgpu_engine::{GfxContext, GuiRenderContext};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;

pub(crate) struct EguiWrapper {
    pub(crate) egui: egui::Context,
    pub(crate) renderer: renderer::RenderPass,
    last_frame: Instant,
    platform: egui_winit::State,
    pub(crate) last_mouse_captured: bool,
    pub(crate) last_kb_captured: bool,
}

impl EguiWrapper {
    pub(crate) fn new(gfx: &mut GfxContext, el: &EventLoopWindowTarget<()>) -> Self {
        let mut egui = egui::Context::default();

        let mut platform = egui_winit::State::new(el);

        let data = std::fs::read("assets/roboto-medium.ttf");
        match data {
            Ok(bold) => {
                let mut defs = FontDefinitions::empty();
                defs.families.insert(
                    egui::FontFamily::Proportional,
                    vec![format!("Roboto Medium")],
                );
                defs.font_data
                    .insert("Roboto Medium".to_string(), FontData::from_owned(bold));
            }
            Err(err) => {
                panic!("font not found: {}", err);
            }
        };

        let renderer = renderer::RenderPass::new(&gfx.device, TextureFormat::Rgba8UnormSrgb, 4);

        Self {
            egui,
            renderer,
            last_frame: Instant::now(),
            last_mouse_captured: false,
            last_kb_captured: false,
            platform,
        }
    }

    pub(crate) fn render(
        &mut self,
        mut gfx: GuiRenderContext<'_, '_>,
        window: &Window,
        hidden: bool,
        ui_render: impl for<'ui> FnOnce(&'ui egui::Context),
    ) {
        let now = Instant::now();
        let delta = now - self.last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame = now;

        let rinput = self.platform.take_egui_input(window);

        let output = self.egui.run(rinput, |ctx| {
            ui_render(ctx);
        });
        let clipped_primitives = self.egui.tessellate(output.shapes);

        let mut rpass = gfx.rpass.take().unwrap();
        for (id, delta) in output.textures_delta.set {
            self.renderer
                .update_texture(&gfx.device, &gfx.queue, id, &delta);
        }
        for id in output.textures_delta.free {
            self.renderer.free_texture(&id);
        }

        if !hidden {
            self.renderer.execute_with_renderpass(
                &mut rpass,
                &clipped_primitives,
                &ScreenDescriptor {
                    size_in_pixels: [gfx.size.0, gfx.size.1],
                    pixels_per_point: 1.0,
                },
            );
        }

        self.platform
            .handle_platform_output(window, &self.egui, output.platform_output);

        self.last_mouse_captured = self.egui.wants_pointer_input();
        self.last_kb_captured = self.egui.wants_keyboard_input();
    }

    pub(crate) fn handle_event(&mut self, e: &winit::event::WindowEvent<'_>) {
        self.platform.on_event(&self.egui, e);
    }
}
