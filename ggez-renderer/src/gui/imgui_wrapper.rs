use gfx_core::{handle::RenderTargetView, memory::Typed};
use gfx_device_gl;
use ggez::graphics;
use ggez::Context;
use imgui::*;
use imgui_gfx_renderer::*;
use scale::gui::Gui;
use specs::World;
use std::time::Instant;

pub struct ImGuiWrapper {
    imgui: imgui::Context,
    renderer: Renderer<gfx_core::format::Rgba8, gfx_device_gl::Resources>,
    last_frame: Instant,
    pub last_mouse_captured: bool,
    pub last_kb_captured: bool,
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

        imgui.io_mut().key_map[imgui::Key::Delete as usize] = 258;
        imgui.io_mut().key_map[imgui::Key::Backspace as usize] = 259;
        imgui.io_mut().key_map[imgui::Key::Enter as usize] = 260;
        imgui.io_mut().key_map[imgui::Key::LeftArrow as usize] = 261;
        imgui.io_mut().key_map[imgui::Key::RightArrow as usize] = 262;
        imgui.io_mut().key_map[imgui::Key::Tab as usize] = 263;

        let renderer = Renderer::init(&mut imgui, &mut *factory, shaders).unwrap();

        Self {
            imgui,
            renderer,
            last_frame: Instant::now(),
            last_mouse_captured: false,
            last_kb_captured: false,
        }
    }

    pub fn render(
        &mut self,
        ctx: &mut Context,
        world: &mut World,
        gui: &mut Gui,
        hidpi_factor: f32,
    ) {
        // Create new frame
        let now = Instant::now();
        let delta = now - self.last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame = now;

        let (draw_width, draw_height) = graphics::drawable_size(ctx);
        self.imgui.io_mut().display_size = [draw_width, draw_height];
        self.imgui.io_mut().display_framebuffer_scale = [hidpi_factor, hidpi_factor];
        self.imgui.io_mut().delta_time = delta_s;

        // Prepare
        let ui: Ui = self.imgui.frame();
        gui.render(&ui, world);
        self.last_mouse_captured = ui.io().want_capture_mouse;
        self.last_kb_captured = ui.io().want_capture_keyboard;

        // Render
        let (factory, _, encoder, _, render_target) = graphics::gfx_objects(ctx);
        let draw_data = ui.render();
        self.renderer
            .render(
                &mut *factory,
                encoder,
                &mut RenderTargetView::new(render_target),
                draw_data,
            )
            .unwrap();

        self.imgui.io_mut().mouse_wheel = 0.0;
        self.imgui.io_mut().keys_down[258] = false;
        self.imgui.io_mut().keys_down[259] = false;
        self.imgui.io_mut().keys_down[260] = false;
        self.imgui.io_mut().keys_down[261] = false;
        self.imgui.io_mut().keys_down[262] = false;
        self.imgui.io_mut().keys_down[263] = false;
    }

    pub fn queue_char(&mut self, c: char) {
        self.imgui.io_mut().add_input_character(c);
    }

    pub fn delete(&mut self) {
        self.imgui.io_mut().keys_down[258] = true;
    }
    pub fn backspace(&mut self) {
        self.imgui.io_mut().keys_down[259] = true;
    }
    pub fn enter(&mut self) {
        self.imgui.io_mut().keys_down[260] = true;
    }
    pub fn left_arrow(&mut self) {
        self.imgui.io_mut().keys_down[261] = true;
    }
    pub fn right_arrow(&mut self) {
        self.imgui.io_mut().keys_down[262] = true;
    }
    pub fn tab(&mut self) {
        self.imgui.io_mut().keys_down[263] = true;
    }

    pub fn update_wheel(&mut self, value: f32) {
        self.imgui.io_mut().mouse_wheel = value;
    }

    pub fn update_mouse_pos(&mut self, x: f32, y: f32) {
        self.imgui.io_mut().mouse_pos = [x, y];
    }

    pub fn update_mouse_down(&mut self, pressed: (bool, bool, bool)) {
        self.imgui.io_mut().mouse_down = [pressed.0, pressed.1, pressed.2, false, false];
    }
}
