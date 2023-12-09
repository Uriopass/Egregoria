use crate::{GfxContext, GuiRenderContext};
use egui::TextureId;
use egui_wgpu::renderer;
use egui_wgpu::renderer::ScreenDescriptor;
use winit::event_loop::EventLoopWindowTarget;

/// EguiWrapper is a wrapper around egui and egui_wgpu
/// It handles the rendering of the UI
pub struct EguiWrapper {
    pub egui: egui::Context,
    pub renderer: renderer::Renderer,
    platform: egui_winit::State,
    pub last_mouse_captured: bool,
    pub last_kb_captured: bool,
    pub to_remove: Vec<TextureId>,
    pub zoom_factor: f32,
}

impl EguiWrapper {
    pub fn new(gfx: &GfxContext, el: &EventLoopWindowTarget<()>) -> Self {
        let egui = egui::Context::default();

        let platform =
            egui_winit::State::new(egui.viewport_id(), el, Some(gfx.size.2 as f32), None);

        let renderer = renderer::Renderer::new(&gfx.device, gfx.fbos.format, None, 1);

        Self {
            egui,
            renderer,
            last_mouse_captured: false,
            last_kb_captured: false,
            platform,
            to_remove: vec![],
            zoom_factor: 1.0,
        }
    }

    pub fn render(
        &mut self,
        gfx: GuiRenderContext<'_, '_>,
        ui_render: impl for<'ui> FnOnce(&'ui egui::Context),
    ) {
        for id in self.to_remove.drain(..) {
            self.renderer.free_texture(&id);
        }

        let rinput = self.platform.take_egui_input(gfx.window);
        self.egui.set_zoom_factor(self.zoom_factor);

        let output = self.egui.run(rinput, |ctx| {
            ui_render(ctx);
        });
        let clipped_primitives = self
            .egui
            .tessellate(output.shapes, self.egui.pixels_per_point());

        //let mut rpass = gfx.rpass.take().unwrap();
        for (id, delta) in output.textures_delta.set {
            self.renderer
                .update_texture(gfx.device, gfx.queue, id, &delta);
        }
        let desc = ScreenDescriptor {
            size_in_pixels: [gfx.size.0, gfx.size.1],
            pixels_per_point: self.egui.pixels_per_point(),
        };
        self.renderer.update_buffers(
            gfx.device,
            gfx.queue,
            gfx.encoder,
            &clipped_primitives,
            &desc,
        );

        self.to_remove = output.textures_delta.free;

        //if !hidden {
        let mut render_pass = gfx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui_render"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: gfx.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.renderer
            .render(&mut render_pass, &clipped_primitives, &desc);
        //}

        self.platform
            .handle_platform_output(gfx.window, &self.egui, output.platform_output);

        self.last_mouse_captured = self.egui.wants_pointer_input();
        self.last_kb_captured = self.egui.wants_keyboard_input();
    }

    pub fn handle_event(&mut self, e: &winit::event::WindowEvent<'_>) {
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
        let _ = self.platform.on_window_event(&self.egui, e);
    }
}
