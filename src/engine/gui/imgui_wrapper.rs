use ggez::graphics;
use ggez::Context;

use gfx_core::factory::Factory;
use gfx_core::{format, texture};
use gfx_core::{handle::RenderTargetView, memory::Typed};
use gfx_device_gl;

use imgui::*;
use imgui_gfx_renderer::*;

use std::time::Instant;

#[derive(Copy, Clone, PartialEq, Debug, Default)]
struct MouseState {
    pos: (i32, i32),
    pressed: (bool, bool, bool),
    wheel: f32,
}

pub struct ImGuiWrapper {
    pub imgui: imgui::Context,
    pub renderer: Renderer<gfx_core::format::Rgba8, gfx_device_gl::Resources>,
    last_frame: Instant,
    mouse_state: MouseState,
    show_popup: bool,
    texture_id: Option<TextureId>,
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
        let mut renderer = Renderer::init(&mut imgui, &mut *factory, shaders).unwrap();

        let rgba_image = image::open(&std::path::Path::new("images/pikachu.png"))
            .unwrap()
            .to_rgba();

        // Load an image as a texture
        let image_dimensions = rgba_image.dimensions();
        let kind = texture::Kind::D2(
            image_dimensions.0 as texture::Size,
            image_dimensions.1 as texture::Size,
            texture::AaMode::Single,
        );
        let (_, texture_view) = factory
            .create_texture_immutable_u8::<format::Srgba8>(
                kind,
                texture::Mipmap::Provided,
                &[rgba_image.into_raw().as_slice()],
            )
            .unwrap();

        // Register the texture with the gfx renderer
        let sampler = factory.create_sampler(texture::SamplerInfo::new(
            texture::FilterMethod::Bilinear,
            texture::WrapMode::Clamp,
        ));
        let texture_id = renderer.textures().insert((texture_view, sampler));

        // Create instace
        Self {
            imgui,
            renderer,
            last_frame: Instant::now(),
            mouse_state: MouseState::default(),
            show_popup: false,
            texture_id: Some(texture_id),
        }
    }

    pub fn render(&mut self, ctx: &mut Context, hidpi_factor: f32) {
        // Update mouse
        self.update_mouse();

        // Create new frame
        let now = Instant::now();
        let delta = now - self.last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame = now;

        let (draw_width, draw_height) = graphics::drawable_size(ctx);
        self.imgui.io_mut().display_size = [draw_width, draw_height];
        self.imgui.io_mut().display_framebuffer_scale = [hidpi_factor, hidpi_factor];
        self.imgui.io_mut().delta_time = delta_s;

        let ui = self.imgui.frame();

        // Various ui things
        {
            // Window with texture
            if let Some(texture_id) = self.texture_id {
                let image = Image::new(&ui, texture_id, [100.0, 100.0]);

                // Window with texture
                ui.window(im_str!("Hello textures"))
                    .size([150.0, 150.0], imgui::Condition::FirstUseEver)
                    .position([300.0, 150.0], imgui::Condition::FirstUseEver)
                    .build(|| {
                        image.build();
                    });
            }

            // Window
            ui.window(im_str!("Hello world"))
                .size([300.0, 600.0], imgui::Condition::FirstUseEver)
                .position([100.0, 100.0], imgui::Condition::FirstUseEver)
                .build(|| {
                    ui.text(im_str!("Hello world!"));
                    ui.text(im_str!("こんにちは世界！"));
                    ui.text(im_str!("This...is...imgui-rs!"));
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(im_str!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos[0],
                        mouse_pos[1]
                    ));

                    if ui.small_button(im_str!("small button")) {
                        println!("Small button clicked");
                    }
                });

            // Popup
            ui.popup(im_str!("popup"), || {
                if ui.menu_item(im_str!("popup menu item 1")).build() {
                    println!("popup menu item 1 clicked");
                }

                if ui.menu_item(im_str!("popup menu item 2")).build() {
                    println!("popup menu item 2 clicked");
                }
            });

            // Menu bar
            ui.main_menu_bar(|| {
                ui.menu(im_str!("Menu 1")).build(|| {
                    if ui.menu_item(im_str!("Item 1.1")).build() {
                        println!("item 1.1 inside menu bar clicked");
                    }

                    ui.menu(im_str!("Item 1.2")).build(|| {
                        if ui.menu_item(im_str!("Item 1.2.1")).build() {
                            println!("item 1.2.1 inside menu bar clicked");
                        }
                        if ui.menu_item(im_str!("Item 1.2.2")).build() {
                            println!("item 1.2.2 inside menu bar clicked");
                        }
                    });
                });

                ui.menu(im_str!("Menu 2")).build(|| {
                    if ui.menu_item(im_str!("Item 2.1")).build() {
                        println!("item 2.1 inside menu bar clicked");
                    }
                });
            });
        }

        if self.show_popup {
            ui.open_popup(im_str!("popup"));
        }

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

    fn update_mouse(&mut self) {
        self.imgui.io_mut().mouse_pos =
            [self.mouse_state.pos.0 as f32, self.mouse_state.pos.1 as f32];

        self.imgui.io_mut().mouse_down = [
            self.mouse_state.pressed.0,
            self.mouse_state.pressed.1,
            self.mouse_state.pressed.2,
            false,
            false,
        ];

        self.imgui.io_mut().mouse_wheel = self.mouse_state.wheel;
        self.mouse_state.wheel = 0.0;
    }

    pub fn update_mouse_pos(&mut self, x: f32, y: f32) {
        self.mouse_state.pos = (x as i32, y as i32);
    }

    pub fn update_mouse_down(&mut self, pressed: (bool, bool, bool)) {
        self.mouse_state.pressed = pressed;

        if pressed.0 {
            self.show_popup = false;
        }
    }

    pub fn open_popup(&mut self) {
        self.show_popup = true;
    }
}
