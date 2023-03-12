use geom::{vec3, Camera, LinearColor, Matrix4, Radians, Vec2, Vec3};
use std::time::Instant;
use wgpu_engine::meshload::load_mesh;
use wgpu_engine::{
    FrameContext, GfxContext, InstancedMesh, InstancedMeshBuilder, Material, MeshInstance,
    MetallicRoughness,
};
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{CursorGrabMode, Window, WindowBuilder};

struct State {
    meshes: Vec<InstancedMesh>,
}

impl State {
    fn new(gfx: &mut GfxContext) -> Self {
        let mesh = load_mesh(gfx, "sphere.glb").unwrap();
        let alb = gfx.material(mesh.material).albedo.clone();

        let mut meshes = vec![];
        for x in 0..=10 {
            for z in 0..=10 {
                let mut c = mesh.clone();

                c.material = gfx.register_material(Material::new_raw(
                    &gfx.device,
                    alb.clone(),
                    MetallicRoughness::Static {
                        metallic: z as f32 / 10.0,
                        roughness: x as f32 / 10.0,
                    },
                ));
                let mut i = InstancedMeshBuilder::new(c);
                i.instances.push(MeshInstance {
                    pos: 2.3 * vec3(x as f32, 0.0, z as f32),
                    dir: Vec3::X,
                    tint: LinearColor::WHITE,
                });
                meshes.push(i.build(gfx).unwrap());
            }
        }

        Self { meshes }
    }

    fn render(&mut self, fc: &mut FrameContext) {
        fc.draw(self.meshes.clone());
    }
}

async fn run(el: EventLoop<()>, window: Window) {
    let mut gfx = GfxContext::new(
        &window,
        window.inner_size().width,
        window.inner_size().height,
    )
    .await;

    gfx.render_params.value_mut().shadow_mapping_resolution = 2048;
    gfx.sun_shadowmap = GfxContext::mk_shadowmap(&gfx.device, 2048);
    gfx.update_simplelit_bg();

    let mut state = State::new(&mut gfx);

    let mut frame: Option<_> = None;
    let mut new_size: Option<PhysicalSize<u32>> = None;
    let mut last_update = Instant::now();

    let sun = vec3(1.0, -1.0, 1.0).normalize();

    let mut camera = Camera::new(vec3(9.0, -30.0, 13.0), 1000.0, 1000.0);
    camera.dist = 0.0;
    camera.pitch = Radians(0.0);
    camera.yaw = Radians(-std::f32::consts::PI / 2.0);

    let mut is_going_forward = false;
    let mut is_going_backward = false;
    let mut is_going_left = false;
    let mut is_going_right = false;
    let mut is_going_up = false;
    let mut is_going_down = false;
    let mut is_going_slow = false;
    let mut is_captured = false;

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
                        is_captured = true;
                    }
                    WindowEvent::CursorLeft {
                        ..
                    } => {
                        let _ = window.set_cursor_grab(CursorGrabMode::None);
                        window.set_cursor_visible(true);
                        is_captured = false;
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
                        is_captured = false;
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                state,
                                scancode,
                                ..
                            },
                        ..
                    } => {
                        let is_pressed = state == winit::event::ElementState::Pressed;
                        match scancode {
                            17 => is_going_forward = is_pressed,
                            31 => is_going_backward = is_pressed,
                            30 => is_going_left = is_pressed,
                            32 => is_going_right = is_pressed,
                            57 => is_going_up = is_pressed,
                            42 => is_going_slow = is_pressed,
                            29 => is_going_down = is_pressed,
                            _ => {}
                        }
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
                        if is_captured {
                            camera.yaw.0 -= 0.001 * (delta.0 as f32);
                            camera.pitch.0 += 0.001 * (delta.1 as f32);
                            camera.pitch.0 = camera.pitch.0.clamp(-1.5, 1.5);
                        }
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

                    let cam_speed = if is_going_slow { 10.0 } else { 30.0 } * delta;
                    if is_captured {
                        if is_going_forward {
                            camera.pos -= camera.dir().xy().z0().try_normalize().unwrap_or(Vec3::ZERO) * cam_speed;
                        }
                        if is_going_backward {
                            camera.pos += camera.dir().xy().z0().try_normalize().unwrap_or(Vec3::ZERO) * cam_speed;
                        }
                        if is_going_left {
                            camera.pos += camera.dir().perp_up().try_normalize().unwrap_or(Vec3::ZERO) * cam_speed;
                        }
                        if is_going_right {
                            camera.pos -= camera.dir().perp_up().try_normalize().unwrap_or(Vec3::ZERO) * cam_speed;
                        }
                        if is_going_up {
                            camera.pos += vec3(0.0, 0.0, 1.0) * cam_speed;
                        }
                        if is_going_down {
                            camera.pos -= vec3(0.0, 0.0, 1.0) * cam_speed;
                        }
                    }

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
                    camera.dist = 300.0;
                    params.sun_shadow_proj = camera.build_sun_shadowmap_matrix(sun, params.shadow_mapping_resolution as f32);
                    camera.dist = 0.0;
                    params.shadow_mapping_resolution = 2048;

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
        .with_inner_size(PhysicalSize::new(
            size.width as f32 * 0.8,
            size.height as f32 * 0.8,
        ))
        .with_title("WGPU Engine Demo for Egregoria")
        .build(&el)
        .expect("Failed to create window");
    beul::execute(run(el, window))
}
