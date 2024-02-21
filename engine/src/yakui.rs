use crate::{GfxContext, GuiRenderContext, Texture};
use std::path::PathBuf;
use std::sync::Arc;
use wgpu::{TextureFormat, TextureViewDescriptor};
use winit::window::Window;
use yakui::font::{Font, FontSettings, Fonts};
use yakui::{TextureId, Yakui};

pub struct YakuiWrapper {
    pub yakui: Yakui,
    pub renderer: yakui_wgpu::YakuiWgpu,
    platform: yakui_winit::YakuiWinit,
    pub zoom_factor: f32,
    pub format: TextureFormat,
    pub blur_bg_texture: TextureId,
}

impl YakuiWrapper {
    pub fn new(gfx: &GfxContext, el: &Window) -> Self {
        let yakui = Yakui::new();

        let fonts = yakui.dom().get_global_or_init(Fonts::default);
        let font = Font::from_bytes(
            include_bytes!("../../assets/font_awesome_solid_900.otf").as_slice(),
            FontSettings::default(),
        )
        .unwrap();
        fonts.add(font, Some("icons"));

        let font = Font::from_bytes(
            include_bytes!("../../assets/SpaceMono-Regular.ttf").as_slice(),
            FontSettings::default(),
        )
        .unwrap();
        fonts.add(font, Some("monospace"));

        let platform = yakui_winit::YakuiWinit::new(el);

        let mut renderer = yakui_wgpu::YakuiWgpu::new(&gfx.device, &gfx.queue);

        let texture_id = renderer.add_texture(
            gfx.null_texture.mip_view(0),
            wgpu::FilterMode::Linear,
            wgpu::FilterMode::Linear,
            wgpu::FilterMode::Linear,
        );

        Self {
            blur_bg_texture: texture_id,
            yakui,
            renderer,
            platform,
            zoom_factor: 1.0,
            format: gfx.fbos.format,
        }
    }

    pub fn load_texture(&mut self, gfx: &mut GfxContext, path: &PathBuf) -> TextureId {
        let tex = gfx.texture(path, "yakui texture");
        self.add_texture(&tex)
    }

    pub fn add_texture(&mut self, tex: &Texture) -> TextureId {
        self.renderer.add_texture(
            Arc::new(tex.texture.create_view(&TextureViewDescriptor::default())),
            wgpu::FilterMode::Linear,
            wgpu::FilterMode::Linear,
            wgpu::FilterMode::Linear,
        )
    }

    pub fn render(&mut self, gfx: &mut GuiRenderContext<'_>, ui_render: impl for<'ui> FnOnce()) {
        self.renderer
            .update_texture(self.blur_bg_texture, gfx.gfx.fbos.ui_blur.mip_view(0));

        self.yakui.set_scale_factor(self.zoom_factor);

        self.yakui.start();
        ui_render();
        {
            profiling::scope!("yakui::finish");
            self.yakui.finish();
        }

        let surface_info = if gfx.gfx.samples > 1 {
            yakui_wgpu::SurfaceInfo {
                format: self.format,
                sample_count: gfx.gfx.samples,
                color_attachment: &gfx.gfx.fbos.color_msaa,
                resolve_target: Some(gfx.view),
            }
        } else {
            yakui_wgpu::SurfaceInfo {
                format: self.format,
                sample_count: 1,
                color_attachment: gfx.view,
                resolve_target: None,
            }
        };

        self.renderer.paint_with_encoder(
            &mut self.yakui,
            &gfx.gfx.device,
            &gfx.gfx.queue,
            gfx.encoder,
            surface_info,
        );
    }

    pub fn handle_event(&mut self, e: &winit::event::Event<()>) -> bool {
        self.platform.handle_event(&mut self.yakui, e)
    }
}
