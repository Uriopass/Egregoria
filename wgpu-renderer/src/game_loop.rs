use crate::engine::{Context, FrameContext, GfxContext};
use crate::geometry::Tesselator;
use crate::rendering::imgui_wrapper::{GuiRenderContext, ImguiWrapper};
use crate::rendering::{CameraHandler, InstancedRender, MeshRenderer, RoadRenderer};
use scale::engine_interaction::{KeyboardInfo, MouseInfo, RenderStats, TimeInfo};
use scale::geometry::{vec2::vec2, Vec2};
use scale::gui::Gui;
use scale::interaction::{FollowEntity, InspectedEntity};
use scale::map_interaction::Itinerary;
use scale::map_model::Map;
use scale::physics::Transform;
use scale::rendering::{Color, LinearColor};
use scale::specs::WorldExt;
use scale::ScaleState;
use std::time::Instant;
use winit::dpi::PhysicalSize;

pub struct State<'a> {
    camera: CameraHandler,
    gui: ImguiWrapper,
    state: ScaleState<'a>,
    last_time: Instant,
    instanced_renderer: InstancedRender,
    road_renderer: RoadRenderer,
    grid: bool,
}

impl<'a> State<'a> {
    pub fn new(ctx: &mut Context) -> Self {
        let camera = CameraHandler::new(ctx.gfx.size.0 as f32, ctx.gfx.size.1 as f32, 3.0);

        let wrapper = ImguiWrapper::new(&mut ctx.gfx);

        let state = scale::ScaleState::setup();

        Self {
            camera,
            gui: wrapper,
            state,
            last_time: Instant::now(),
            instanced_renderer: InstancedRender::new(&mut ctx.gfx),
            road_renderer: RoadRenderer::new(&mut ctx.gfx),
            grid: true,
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        let delta = self.last_time.elapsed().as_secs_f64();
        self.last_time = Instant::now();

        self.manage_time(delta);

        self.manage_io(ctx);

        self.camera.easy_camera_movement(
            ctx,
            delta as f32,
            !self.gui.last_mouse_captured,
            !self.gui.last_kb_captured,
        );

        self.state.run();

        self.manage_entity_follow();
        self.camera.update(ctx);
    }

    pub fn render(&mut self, ctx: &mut FrameContext) {
        let start = Instant::now();

        let mut tess = self.camera.culled_tesselator();
        // Render grid
        if self.grid && self.camera.zoom() > 3.0 {
            let gray_maj = (self.camera.zoom() / 40.0).min(0.2);
            let gray_min = gray_maj / 2.0;
            if self.camera.zoom() > 6.0 {
                tess.draw_grid(1.0, Color::new(gray_min, gray_min, gray_min, 1.0));
            }
            tess.draw_grid(10.0, Color::new(gray_maj, gray_maj, gray_maj, 1.0));
        }

        let time: TimeInfo = *self.state.world.read_resource::<TimeInfo>();
        self.road_renderer.render(
            &mut self.state.world.write_resource::<Map>(),
            time.time_seconds,
            &mut tess,
            ctx,
        );

        self.instanced_renderer.render(&mut self.state.world, ctx);

        MeshRenderer::render(&mut self.state.world, &mut tess);

        debug_pathfinder(&mut tess, &self.state.world);

        for (order, col) in scale::utils::debugdraw::PERSISTENT_DEBUG_ORDERS
            .lock()
            .unwrap() // Unwrap ok: Mutex lives in main thread
            .iter()
            .copied()
            .chain(
                scale::utils::debugdraw::DEBUG_ORDERS
                    .lock()
                    .unwrap() // Unwrap ok: Mutex lives in main thread
                    .drain(..),
            )
        {
            tess.color = col.into();
            use scale::utils::debugdraw::DebugOrder::*;
            match order {
                Point { pos, size } => {
                    tess.draw_circle(pos, 3.0, size);
                }
                Line { from, to } => {
                    tess.draw_line(from, to, 3.0);
                }
            }
        }

        if let Some(x) = tess.meshbuilder.build(ctx.gfx) {
            ctx.draw(x)
        }

        self.state.world.write_resource::<RenderStats>().render_time =
            start.elapsed().as_secs_f32();
    }

    pub fn render_gui(&mut self, ctx: GuiRenderContext) {
        let mut gui = (*self.state.world.read_resource::<Gui>()).clone();
        self.gui.render(ctx, &mut self.state.world, &mut gui);
        *self.state.world.write_resource::<Gui>() = gui;
    }

    fn manage_time(&mut self, delta: f64) {
        const MAX_TIMESTEP: f64 = 1.0 / 30.0;
        let delta = delta.min(MAX_TIMESTEP);

        let mut time = self.state.world.write_resource::<TimeInfo>();
        time.delta = delta as f32;
        time.time += time.delta as f64;
        time.time_seconds = time.time as u64;
    }

    fn manage_entity_follow(&mut self) {
        if !self
            .state
            .world
            .read_resource::<MouseInfo>()
            .just_pressed
            .is_empty()
        {
            self.state.world.write_resource::<FollowEntity>().0.take();
        }

        if let Some(e) = self.state.world.read_resource::<FollowEntity>().0 {
            if let Some(pos) = self
                .state
                .world
                .read_component::<Transform>()
                .get(e)
                .map(|x| x.position())
            {
                self.camera.camera.position = [pos.x, pos.y].into();
            }
        }
    }

    fn manage_io(&mut self, ctx: &Context) {
        *self.state.world.write_resource::<KeyboardInfo>() = ctx.input.keyboard.clone();
        *self.state.world.write_resource::<MouseInfo>() = ctx.input.mouse.clone();

        if self.gui.last_kb_captured {
            let kb: &mut KeyboardInfo = &mut self.state.world.write_resource::<KeyboardInfo>();
            kb.just_pressed.clear();
            kb.is_pressed.clear();
        }

        if self.gui.last_mouse_captured {
            let mouse: &mut MouseInfo = &mut self.state.world.write_resource::<MouseInfo>();
            mouse.just_pressed.clear();
            mouse.buttons.clear();
            mouse.wheel_delta = 0.0;
        }
    }

    pub fn event(&mut self, gfx: &GfxContext, event: &winit::event::Event<()>) {
        self.gui.handle_event(gfx, event);
    }

    pub fn resized(&mut self, ctx: &mut Context, size: PhysicalSize<u32>) {
        self.camera
            .resize(ctx, size.width as f32, size.height as f32);
    }

    pub fn unproject(&mut self, pos: Vec2) -> Vec2 {
        self.camera.unproject_mouse_click(pos)
    }
}

#[allow(dead_code)]
fn debug_pathfinder(tess: &mut Tesselator, world: &scale::specs::World) -> Option<()> {
    let map: &Map = &world.read_resource::<Map>();
    let selected = world.read_resource::<InspectedEntity>().e?;
    let pos = world.read_storage::<Transform>().get(selected)?.position();

    let stor = world.read_storage::<Itinerary>();
    let itinerary = stor.get(selected)?;

    tess.color = LinearColor::GREEN;
    tess.draw_polyline(&itinerary.local_path(), 1.0, 1.0);

    if let Some(p) = itinerary.get_point() {
        tess.draw_stroke(p, pos, 1.0, 1.0);
    }

    if let scale::map_interaction::ItineraryKind::Route(r) = itinerary.kind() {
        tess.color = LinearColor::RED;
        for l in &r.reversed_route {
            tess.draw_polyline(l.raw_points(map).as_slice(), 1.0, 3.0);
        }
        tess.color = LinearColor::MAGENTA;
        tess.draw_circle(r.end_pos, 1.0, 1.0);
    }
    Some(())
}

#[allow(dead_code)]
fn debug_rays(tess: &mut Tesselator, world: &scale::specs::World) {
    let time = world.read_resource::<TimeInfo>();
    let time = time.time * 0.2;
    let c = time.cos() as f32;
    let s = time.sin() as f32;

    let r = scale::geometry::intersections::Ray {
        from: 10.0 * vec2(c, s),
        dir: vec2(
            (time * 2.3 + 1.0).cos() as f32,
            (time * 2.3 + 1.0).sin() as f32,
        ),
    };

    let r2 = scale::geometry::intersections::Ray {
        from: 10.0 * vec2((time as f32 * 1.5 + 3.0).cos(), s * 2.0),
        dir: vec2(c, -s),
    };

    tess.color = LinearColor::WHITE;
    tess.draw_line(r.from, r.from + r.dir * 50.0, 0.5);
    tess.draw_line(r2.from, r2.from + r2.dir * 50.0, 0.5);

    let inter = scale::geometry::intersections::intersection_point(r, r2);
    if let Some(v) = inter {
        tess.color = LinearColor::RED;

        tess.draw_circle(v, 0.5, 2.0);
    }
}
