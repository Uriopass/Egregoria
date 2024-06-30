use crate::{GfxContext, GuiRenderContext};
use egui::TextureId;
use egui_wgpu::{Renderer, ScreenDescriptor};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;

/// EguiWrapper is a wrapper around egui and egui_wgpu
/// It handles the rendering of the UI
pub struct EguiWrapper {
    pub renderer: Renderer,
    pub platform: egui_winit::State,
    pub last_mouse_captured: bool,
    pub last_kb_captured: bool,
    pub to_remove: Vec<TextureId>,
    pub zoom_factor: f32,
}

impl EguiWrapper {
    pub fn new(gfx: &GfxContext, el: &EventLoopWindowTarget<()>) -> Self {
        let egui = egui::Context::default();

        let viewport_id = egui.viewport_id();
        let platform = egui_winit::State::new(egui, viewport_id, el, Some(gfx.size.2 as f32), None);

        let renderer = Renderer::new(&gfx.device, gfx.fbos.format, None, 1);

        Self {
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
        gfx: GuiRenderContext<'_>,
        ui_render: impl for<'ui> FnOnce(&'ui egui::Context),
    ) {
        for id in self.to_remove.drain(..) {
            self.renderer.free_texture(&id);
        }

        let rinput = self.platform.take_egui_input(&gfx.gfx.window);
        let egui = self.platform.egui_ctx();
        egui.set_zoom_factor(self.zoom_factor);

        let output = egui.run(rinput, |ctx| {
            ui_render(ctx);
        });
        let clipped_primitives = egui.tessellate(output.shapes, egui.pixels_per_point());

        //let mut rpass = gfx.rpass.take().unwrap();
        for (id, delta) in output.textures_delta.set {
            self.renderer
                .update_texture(&gfx.gfx.device, &gfx.gfx.queue, id, &delta);
        }
        let desc = ScreenDescriptor {
            size_in_pixels: [gfx.size.0, gfx.size.1],
            pixels_per_point: egui.pixels_per_point(),
        };
        self.renderer.update_buffers(
            &gfx.gfx.device,
            &gfx.gfx.queue,
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

        self.platform
            .handle_platform_output(&gfx.gfx.window, output.platform_output);

        self.last_mouse_captured = self.platform.egui_ctx().wants_pointer_input();
        self.last_kb_captured = self.platform.egui_ctx().wants_keyboard_input();
    }

    pub fn handle_event(&mut self, window: &Window, e: &winit::event::WindowEvent) {
        //if let winit::event::WindowEvent::KeyboardInput {
        //    input:
        //        winit::event::KeyboardInput {
        //            virtual_keycode: Some(winit::event::VirtualKeyCode::Tab),
        //            ..
        //        },
        //    ..
        //} = e
        //{
        //    return;
        //}
        let _ = self.platform.on_window_event(window, e);
    }
}
