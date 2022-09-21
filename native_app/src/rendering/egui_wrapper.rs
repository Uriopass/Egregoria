use egui::{FontData, FontDefinitions, TextureId};
use egui_wgpu::renderer;
use egui_wgpu::renderer::ScreenDescriptor;
use wgpu_engine::{GfxContext, GuiRenderContext};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;

pub(crate) struct EguiWrapper {
    pub(crate) egui: egui::Context,
    pub(crate) renderer: renderer::RenderPass,
    platform: egui_winit::State,
    pub(crate) last_mouse_captured: bool,
    pub(crate) last_kb_captured: bool,
    pub to_remove: Vec<TextureId>,
}

impl EguiWrapper {
    pub(crate) fn new(gfx: &mut GfxContext, el: &EventLoopWindowTarget<()>) -> Self {
        let egui = egui::Context::default();

        let platform = egui_winit::State::new(el);

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

        let renderer = renderer::RenderPass::new(&gfx.device, gfx.fbos.format, 1);

        Self {
            egui,
            renderer,
            last_mouse_captured: false,
            last_kb_captured: false,
            platform,
            to_remove: vec![],
        }
    }

    pub(crate) fn render(
        &mut self,
        gfx: GuiRenderContext<'_, '_>,
        window: &Window,
        hidden: bool,
        ui_render: impl for<'ui> FnOnce(&'ui egui::Context),
    ) {
        for id in self.to_remove.drain(..) {
            self.renderer.free_texture(&id);
        }

        let mut rinput = self.platform.take_egui_input(window);
        rinput.screen_rect = Some(egui::Rect::from_min_size(
            Default::default(),
            egui::vec2(gfx.size.0 as f32, gfx.size.1 as f32),
        ));

        let output = self.egui.run(rinput, |ctx| {
            ui_render(ctx);
        });
        let clipped_primitives = self.egui.tessellate(output.shapes);

        //let mut rpass = gfx.rpass.take().unwrap();
        for (id, delta) in output.textures_delta.set {
            self.renderer
                .update_texture(gfx.device, gfx.queue, id, &delta);
        }
        let desc = ScreenDescriptor {
            size_in_pixels: [gfx.size.0, gfx.size.1],
            pixels_per_point: 1.0,
        };
        self.renderer
            .update_buffers(gfx.device, gfx.queue, &clipped_primitives, &desc);

        self.to_remove = output.textures_delta.free;

        if !hidden {
            self.renderer
                .execute(gfx.encoder, gfx.view, &clipped_primitives, &desc, None);
            /*
            self.renderer
                .execute_with_renderpass(&mut rpass, &clipped_primitives, &desc);*/
        }

        self.platform
            .handle_platform_output(window, &self.egui, output.platform_output);

        self.last_mouse_captured = self.egui.wants_pointer_input();
        self.last_kb_captured = self.egui.wants_keyboard_input();
    }

    pub(crate) fn handle_event(&mut self, e: &winit::event::WindowEvent<'_>) {
        if let winit::event::WindowEvent::KeyboardInput {
            input:
                winit::event::KeyboardInput {
                    virtual_keycode: Some(winit::event::VirtualKeyCode::Tab),
                    ..
                },
            ..
        } = e
        {
            return;
        }
        self.platform.on_event(&self.egui, e);
    }
}
