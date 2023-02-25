use crate::audio::AudioContext;
use crate::game_loop;
use crate::init::SOUNDS_LIST;
use crate::input::InputContext;
use egregoria::utils::time::GameTime;
use geom::{vec2, vec3, LinearColor};
use std::time::Instant;
use wgpu_engine::GfxContext;
use winit::window::Window;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

pub(crate) struct Context {
    pub(crate) gfx: GfxContext,
    pub(crate) input: InputContext,
    pub(crate) audio: AudioContext,
    pub(crate) window: Window,
    pub(crate) el: Option<EventLoop<()>>,
    pub(crate) delta: f32,
}

impl Context {
    pub(crate) async fn new(el: EventLoop<()>, window: Window) -> Self {
        let gfx = GfxContext::new(
            &window,
            window.inner_size().width,
            window.inner_size().height,
        )
        .await;
        let input = InputContext::default();
        let mut audio = AudioContext::new();

        audio.preload(
            SOUNDS_LIST
                .files()
                .flat_map(|x| x.path().file_name())
                .flat_map(|x| x.to_str())
                .map(|x| x.trim_end_matches(".ogg")),
        );

        Self {
            gfx,
            input,
            audio,
            window,
            el: Some(el),
            delta: 0.0,
        }
    }

    pub(crate) fn start(mut self, mut state: game_loop::State) {
        let mut frame: Option<_> = None;
        let mut new_size: Option<PhysicalSize<u32>> = None;
        let mut last_update = Instant::now();

        self.el.take().unwrap().run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent { event, .. } => {
                    state.event(&event);
                    self.input.handle(&event);

                    match event {
                        WindowEvent::Resized(physical_size) => {
                            log::info!("resized: {:?}", physical_size);
                            new_size = Some(physical_size);
                            frame.take();
                        }
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        _ => (),
                    }
                }
                Event::MainEventsCleared => match frame.take() {
                    None => {
                        if let Some(new_size) = new_size.take() {
                            self.gfx.resize(new_size.width, new_size.height);
                            state.resized(&mut self, new_size);
                            self.gfx.update_sc = false;
                        }

                        let size = self.gfx.size;
                        if self.gfx.update_sc {
                            self.gfx.update_sc = false;
                            self.gfx.resize(size.0, size.1);
                            state.resized(
                                &mut self,
                                PhysicalSize {
                                    width: size.0,
                                    height: size.1,
                                },
                            );
                        }

                        match self.gfx.surface.get_current_texture() {
                            Ok(swapchainframe) => {
                                frame = Some(swapchainframe);
                            }
                            Err(wgpu_engine::wgpu::SurfaceError::Outdated)
                            | Err(wgpu_engine::wgpu::SurfaceError::Lost)
                            | Err(wgpu_engine::wgpu::SurfaceError::Timeout) => {
                                self.gfx.resize(size.0, size.1);
                                state.resized(&mut self, PhysicalSize::new(size.0, size.1));
                                log::error!("swapchain has been lost or is outdated, recreating before retrying");
                            }
                            Err(e) => panic!("error getting swapchain: {e}"),
                        };
                    }
                    Some(_) if new_size.is_some() => {}
                    Some(sco) => {
                        profiling::finish_frame!();
                        profiling::scope!("frame");
                        let d = last_update.elapsed();
                        last_update = Instant::now();
                        self.delta = d.as_secs_f32();
                        state.update(&mut self);

                        let t = std::f32::consts::TAU
                            * (self.gfx.render_params.value().time - 8.0 * GameTime::HOUR as f32)
                            / GameTime::DAY as f32;

                        let sun = vec3(t.cos(), t.sin() * 0.5, t.sin() + 0.5).normalize();

                        let params = self.gfx.render_params.value_mut();
                        params.time_always = (params.time_always + self.delta) % 3600.0;
                        params.sun_col = sun.z.max(0.0).sqrt().sqrt() * LinearColor::new(1.0, 0.95 + sun.z * 0.05, 0.95 + sun.z * 0.05, 1.0);
                        params.cam_pos = state.camera.camera.eye();
                        params.cam_dir = -state.camera.camera.dir();
                        params.sun = sun;
                        params.viewport = vec2(self.gfx.size.0 as f32, self.gfx.size.1 as f32);
                        params.sun_shadow_proj =
                            state.camera.camera.build_sun_shadowmap_matrix(sun, params.shadow_mapping_enabled as f32);
                        let c = egregoria::config();
                        params.grass_col = c.grass_col.into();
                        params.sand_col = c.sand_col.into();
                        params.sea_col = c.sea_col.into();
                        params.ssao_strength = c.ssao_strength;
                        params.ssao_radius = c.ssao_radius;
                        params.ssao_falloff = c.ssao_falloff;
                        params.ssao_base = c.ssao_base;
                        params.ssao_samples = c.ssao_samples;
                        drop(c);

                        let (mut enc, view) = self.gfx.start_frame(&sco);
                        self.gfx.render_objs(&mut enc, &view, |fc| state.render(fc));

                        let window = &self.window;
                        self.gfx
                            .render_gui(&mut enc, &view, |gctx| state.render_gui(window, gctx));
                        self.gfx.finish_frame(enc);
                        sco.present();

                        self.input.end_frame();
                    }
                },
                _ => (),
            }
        })
    }
}
