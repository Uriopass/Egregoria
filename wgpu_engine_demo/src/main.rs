use geom::{vec3, Camera, LinearColor, Matrix4, Vec2};
use std::time::Instant;
use wgpu_engine::meshload::load_mesh;
use wgpu_engine::{FrameContext, GfxContext, Mesh};
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{CursorGrabMode, Window, WindowBuilder};

struct State {
    stop_sign: Mesh,
}

impl State {
    fn new(gfx: &mut GfxContext) -> Self {
        Self {
            stop_sign: load_mesh(gfx, "flour_factory.glb").unwrap(),
        }
    }

    fn render(&mut self, fc: &mut FrameContext) {
        fc.draw(self.stop_sign.clone());
    }
}

async fn run(el: EventLoop<()>, window: Window) {
    let mut gfx = GfxContext::new(
        &window,
        window.inner_size().width,
        window.inner_size().height,
    )
    .await;

    let mut state = State::new(&mut gfx);

    let mut frame: Option<_> = None;
    let mut new_size: Option<PhysicalSize<u32>> = None;
    let mut last_update = Instant::now();

    let sun = vec3(1.0, 1.0, 1.0).normalize();

    let mut camera = Camera::new(vec3(0.0, 0.0, 0.0), 1000.0, 1000.0);

    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        log::info!("resized: {:?}", physical_size);
                        new_size = Some(physical_size);
                        frame.take();
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::MouseInput {
                        state: winit::event::ElementState::Pressed,
                        button: winit::event::MouseButton::Left,
                        ..
                    } => {
                        let _ = window.set_cursor_grab(CursorGrabMode::Confined);
                        window.set_cursor_visible(false);
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                state: winit::event::ElementState::Pressed,
                                virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => {
                        let _ = window.set_cursor_grab(CursorGrabMode::None);
                        window.set_cursor_visible(true);
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                state: winit::event::ElementState::Pressed,
                                scancode: 17,
                                ..
                            },
                        ..
                    } => {
                        camera.pos += 0.1 * camera.dir();
                    }
                    _ => (),
                }
            }
            Event::DeviceEvent {
                event,
                ..
            } => {
                match event {
                    DeviceEvent::MouseMotion {
                        delta
                    } => {
                        camera.yaw.0   -= 0.001 * (delta.0 as f32);
                        camera.pitch.0 += 0.001 * (delta.1 as f32);
                    }
                    _ => {}
                }
            }
            Event::MainEventsCleared => match frame.take() {
                None => {
                    if let Some(new_size) = new_size.take() {
                        gfx.resize(new_size.width, new_size.height);
                        gfx.update_sc = false;
                        camera.set_viewport(new_size.width as f32, new_size.height as f32);
                    }

                    let size = gfx.size;
                    if gfx.update_sc {
                        gfx.update_sc = false;
                        gfx.resize(size.0, size.1);
                    }

                    match gfx.surface.get_current_texture() {
                        Ok(swapchainframe) => {
                            frame = Some(swapchainframe);
                        }
                        Err(wgpu_engine::wgpu::SurfaceError::Outdated)
                        | Err(wgpu_engine::wgpu::SurfaceError::Lost)
                        | Err(wgpu_engine::wgpu::SurfaceError::Timeout) => {
                            gfx.resize(size.0, size.1);
                            log::error!("swapchain has been lost or is outdated, recreating before retrying");
                        }
                        Err(e) => panic!("error getting swapchain: {e}"),
                    };
                }
                Some(_) if new_size.is_some() => {}
                Some(sco) => {
                    let d = last_update.elapsed();
                    last_update = Instant::now();
                    let delta = d.as_secs_f32();

                    let viewproj = camera.build_view_projection_matrix();
                    let inv_viewproj = viewproj.invert().unwrap_or_else(Matrix4::zero);

                    gfx.set_proj(viewproj);
                    gfx.set_inv_proj(inv_viewproj);

                    let params = gfx.render_params.value_mut();
                    params.time_always = (params.time_always + delta) % 3600.0;
                    params.sun_col = sun.z.max(0.0).sqrt().sqrt() * LinearColor::new(1.0, 0.95 + sun.z * 0.05, 0.95 + sun.z * 0.05, 1.0);
                    params.cam_pos = camera.eye();
                    params.cam_dir = camera.dir();
                    params.sun = sun;
                    params.viewport = Vec2::new(gfx.size.0 as f32, gfx.size.1 as f32);
                    params.sun_shadow_proj = camera.build_sun_shadowmap_matrix(sun, params.shadow_mapping_enabled as f32);
                    params.ssao_strength = inline_tweak::tweak!(0.64);
                    params.ssao_radius = inline_tweak::tweak!(0.343);
                    params.ssao_falloff = inline_tweak::tweak!(0.00008);
                    params.ssao_base = inline_tweak::tweak!(0.01);
                    params.ssao_samples = inline_tweak::tweak!(8);

                    let (mut enc, view) = gfx.start_frame(&sco);
                    gfx.render_objs(&mut enc, &view, |fc| state.render(fc));
                    gfx.finish_frame(enc);
                    sco.present();
                }
            },
            _ => (),
        }
    })
}

fn main() {
    let el = EventLoop::new();
    common::logger::MyLog::init();

    let size = match el.primary_monitor() {
        Some(monitor) => monitor.size(),
        None => el.available_monitors().next().unwrap().size(),
    };

    let wb = WindowBuilder::new();

    let window;
    #[cfg(target_os = "windows")]
    {
        use winit::platform::windows::WindowBuilderExtWindows;
        window = wb.with_drag_and_drop(false);
    }
    #[cfg(not(target_os = "windows"))]
    {
        window = wb;
    }
    let window = window
        .with_inner_size(winit::dpi::PhysicalSize::new(
            size.width as f32 * 0.8,
            size.height as f32 * 0.8,
        ))
        .with_title(format!("WGPU Engine Demo for Egregoria"))
        .build(&el)
        .expect("Failed to create window");
    beul::execute(run(el, window))
}
