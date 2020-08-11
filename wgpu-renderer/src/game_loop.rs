use crate::engine::{Context, FrameContext, GfxContext};
use crate::rendering::imgui_wrapper::{GuiRenderContext, ImguiWrapper};
use crate::rendering::{CameraHandler, InstancedRender, MeshRenderer, RoadRenderer};
use egregoria::engine_interaction::{KeyboardInfo, MouseInfo, RenderStats, TimeInfo};
use egregoria::gui::Gui;
use egregoria::interaction::FollowEntity;
use egregoria::physics::Transform;
use egregoria::rendering::immediate::{ImmediateDraw, ImmediateOrder};
use egregoria::rendering::Color;
use egregoria::specs::WorldExt;
use egregoria::EgregoriaState;
use geom::Vec2;
use map_model::Map;
use std::time::Instant;
use winit::dpi::PhysicalSize;

pub struct State {
    camera: CameraHandler,
    gui: ImguiWrapper,
    state: EgregoriaState,
    last_time: Instant,
    instanced_renderer: InstancedRender,
    road_renderer: RoadRenderer,
    grid: bool,
}

impl State {
    pub fn new(ctx: &mut Context) -> Self {
        let camera = CameraHandler::new(ctx.gfx.size.0 as f32, ctx.gfx.size.1 as f32, 3.0);

        let wrapper = ImguiWrapper::new(&mut ctx.gfx);

        let state = egregoria::EgregoriaState::setup();

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

        if !self.gui.last_mouse_captured {
            self.state.world.write_resource::<MouseInfo>().unprojected =
                self.unproject(ctx.input.mouse.screen);
        }

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
            let gray_min = gray_maj * 0.5;
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

        {
            let objs = crate::debug::DEBUG_OBJS.lock().unwrap();
            for (val, _, obj) in &*objs {
                if *val {
                    obj(&mut tess, &self.state.world);
                }
            }
        }

        let immediate = &mut *self.state.world.write_resource::<ImmediateDraw>();
        for (order, col) in immediate
            .persistent_orders
            .iter()
            .copied()
            .chain(immediate.orders.drain(..))
        {
            tess.color = col.into();
            match order {
                ImmediateOrder::Circle { pos, size } => {
                    tess.draw_circle(pos, 3.0, size);
                }
                ImmediateOrder::Line { from, to } => {
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
        const MAX_TIMESTEP: f64 = 1.0 / 10.0;
        let mut time = self.state.world.write_resource::<TimeInfo>();

        let delta = (delta * time.time_speed as f64).min(MAX_TIMESTEP);
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

    pub fn unproject(&self, pos: Vec2) -> Vec2 {
        self.camera.unproject_mouse_click(pos)
    }
}
