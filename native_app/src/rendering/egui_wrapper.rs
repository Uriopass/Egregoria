use egui::TextureId;
use egui_wgpu::renderer;
use egui_wgpu::renderer::ScreenDescriptor;
use wgpu_engine::{GfxContext, GuiRenderContext};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;

pub(crate) struct EguiWrapper {
    pub(crate) egui: egui::Context,
    pub(crate) renderer: renderer::Renderer,
    platform: egui_winit::State,
    pub(crate) last_mouse_captured: bool,
    pub(crate) last_kb_captured: bool,
    pub to_remove: Vec<TextureId>,
}

impl EguiWrapper {
    pub(crate) fn new(gfx: &mut GfxContext, el: &EventLoopWindowTarget<()>) -> Self {
        let egui = egui::Context::default();

        let platform = egui_winit::State::new(el);

        let renderer = renderer::Renderer::new(&gfx.device, gfx.fbos.format, None, 1);

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
        pixels_per_point: f32,
        ui_render: impl for<'ui> FnOnce(&'ui egui::Context),
    ) {
        for id in self.to_remove.drain(..) {
            self.renderer.free_texture(&id);
        }

        let mut rinput = self.platform.take_egui_input(window);
        rinput.screen_rect = Some(egui::Rect::from_min_size(
            Default::default(),
            egui::vec2(
                gfx.size.0 as f32 / pixels_per_point,
                gfx.size.1 as f32 / pixels_per_point,
            ),
        ));
        rinput.pixels_per_point = Some(pixels_per_point);

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
            pixels_per_point,
        };
        self.renderer.update_buffers(
            gfx.device,
            gfx.queue,
            gfx.encoder,
            &clipped_primitives,
            &desc,
        );

        self.to_remove = output.textures_delta.free;

        if !hidden {
            let mut render_pass =
                gfx.encoder
                    .begin_render_pass(&wgpu_engine::wgpu::RenderPassDescriptor {
                        color_attachments: &[Some(wgpu_engine::wgpu::RenderPassColorAttachment {
                            view: gfx.view,
                            resolve_target: None,
                            ops: wgpu_engine::wgpu::Operations {
                                load: wgpu_engine::wgpu::LoadOp::Load,
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                        label: Some("egui_render"),
                    });

            self.renderer
                .render(&mut render_pass, &clipped_primitives, &desc);
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
        let _ = self.platform.on_event(&self.egui, e);
    }
}
