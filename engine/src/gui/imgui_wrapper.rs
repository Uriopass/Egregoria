use ggez::graphics;
use ggez::Context;

use gfx_core::{handle::RenderTargetView, memory::Typed};
use gfx_device_gl;

use imgui::*;
use imgui_gfx_renderer::*;

use specs::World;
use std::time::Instant;

pub trait Gui: Clone + Send + Sync {
    fn render(&self, ui: &Ui, world: &mut World);
}

pub struct ImGuiWrapper {
    imgui: imgui::Context,
    renderer: Renderer<gfx_core::format::Rgba8, gfx_device_gl::Resources>,
    last_frame: Instant,
    pub last_mouse_wheel: f32,
    show_popup: bool,
}

impl ImGuiWrapper {
    pub fn new(ctx: &mut Context) -> Self {
        // Create the imgui object
        let mut imgui = imgui::Context::create();
        let (factory, gfx_device, _, _, _) = graphics::gfx_objects(ctx);

        // Shaders
        let shaders = {
            let version = gfx_device.get_info().shading_language;
            if version.is_embedded {
                if version.major >= 3 {
                    Shaders::GlSlEs300
                } else {
                    Shaders::GlSlEs100
                }
            } else if version.major >= 4 {
                Shaders::GlSl400
            } else if version.major >= 3 {
                Shaders::GlSl130
            } else {
                Shaders::GlSl110
            }
        };

        // Renderer
        let renderer = Renderer::init(&mut imgui, &mut *factory, shaders).unwrap();

        // Create instace
        Self {
            imgui,
            renderer,
            last_frame: Instant::now(),
            last_mouse_wheel: 0.0,
            show_popup: false,
        }
    }

    pub fn render<G: Gui>(
        &mut self,
        ctx: &mut Context,
        world: &mut World,
        gui: G,
        hidpi_factor: f32,
    ) {
        // Update mouse
        self.update_mouse(ctx);

        // Create new frame
        let now = Instant::now();
        let delta = now - self.last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame = now;

        let (draw_width, draw_height) = graphics::drawable_size(ctx);
        self.imgui.io_mut().display_size = [draw_width, draw_height];
        self.imgui.io_mut().display_framebuffer_scale = [hidpi_factor, hidpi_factor];
        self.imgui.io_mut().delta_time = delta_s;

        let ui: Ui = self.imgui.frame();

        gui.render(&ui, world);

        // Render
        let (factory, _, encoder, _, render_target) = graphics::gfx_objects(ctx);
        let draw_data = ui.render();
        self.renderer
            .render(
                &mut *factory,
                encoder,
                &mut RenderTargetView::new(render_target.clone()),
                draw_data,
            )
            .unwrap();
    }

    fn update_mouse(&mut self, ctx: &mut Context) {
        let pos = ggez::input::mouse::position(ctx);
        self.imgui.io_mut().mouse_pos = [pos.x, pos.y];

        self.imgui.io_mut().mouse_down = [
            ggez::input::mouse::button_pressed(ctx, ggez::input::mouse::MouseButton::Left),
            ggez::input::mouse::button_pressed(ctx, ggez::input::mouse::MouseButton::Right),
            ggez::input::mouse::button_pressed(ctx, ggez::input::mouse::MouseButton::Middle),
            false,
            false,
        ];

        self.imgui.io_mut().mouse_wheel = self.last_mouse_wheel;
        self.last_mouse_wheel = 0.0;
    }

    pub fn open_popup(&mut self) {
        self.show_popup = true;
    }
}
