use crate::engine::{Context, Drawable, FrameContext, GfxContext};
use crate::geometry::Tesselator;
use crate::rendering::imgui_wrapper::ImguiWrapper;
use crate::rendering::{CameraHandler, InstancedRender, RoadRenderer};
use cgmath::Vector2;
use scale::engine_interaction::{KeyboardInfo, MouseInfo, RenderStats, TimeInfo};
use scale::gui::Gui;
use scale::interaction::FollowEntity;
use scale::map_model::{Map, MapUIState};
use scale::physics::Transform;
use scale::rendering::Color;
use scale::specs::RunNow;
use scale::specs::WorldExt;
use std::fs::File;
use std::io::Read;
use std::time::Instant;
use winit::dpi::PhysicalSize;

pub struct State<'a> {
    camera: CameraHandler,
    gui: ImguiWrapper,
    world: scale::specs::World,
    dispatcher: scale::specs::Dispatcher<'a, 'a>,
    time_sync: f64,
    last_time: Instant,
    instanced_renderer: InstancedRender,
    road_renderer: RoadRenderer,
    grid: bool,
}

const TIME_STEP: f64 = 1.0 / 50.0;

impl<'a> State<'a> {
    pub fn new(ctx: &mut Context) -> Self {
        let camera = CameraHandler::new(ctx.gfx.size.0 as f32, ctx.gfx.size.1 as f32, 10.0);

        let mut buf = vec![];
        File::open("resources/music.mp3")
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        /*let source = Decoder::new(std::io::Cursor::new(buf)).unwrap();
                ctx.audio
                    .play_sound(source.fade_in(Duration::new(1, 0)).repeat_infinite(), 0.02);
        */
        let wrapper = ImguiWrapper::new(&mut ctx.gfx);

        let mut world = scale::specs::World::empty();
        let dispatcher = scale::setup(&mut world);

        Self {
            camera,
            gui: wrapper,
            world,
            dispatcher,
            time_sync: 0.0,
            last_time: Instant::now(),
            instanced_renderer: InstancedRender::new(&mut ctx.gfx),
            road_renderer: RoadRenderer::new(),
            grid: true,
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        let delta = (Instant::now() - self.last_time).as_secs_f64();

        self.last_time = Instant::now();

        self.manage_timestep(delta);

        self.manage_io(ctx);

        self.dispatcher.run_now(&self.world);
        self.world.maintain();

        self.camera.easy_camera_movement(
            ctx,
            delta as f32,
            !self.gui.last_mouse_captured,
            !self.gui.last_kb_captured,
        );
        self.manage_entity_follow();
        self.camera.update(ctx);

        self.world.write_resource::<RenderStats>().update_time =
            (Instant::now() - self.last_time).as_secs_f32();
    }

    pub fn render(&mut self, ctx: &mut FrameContext) {
        let time: TimeInfo = *self.world.read_resource::<TimeInfo>();

        let mut tess = self.camera.culled_tesselator();
        // Render grid
        if self.grid && self.camera.zoom() > 3.0 {
            let gray_maj = (self.camera.zoom() / 40.0).min(0.2);
            let gray_min = gray_maj / 2.0;
            if self.camera.zoom() > 6.0 {
                self.draw_grid(
                    &mut tess,
                    1.0,
                    Color::new(gray_min, gray_min, gray_min, 1.0),
                );
            }
            self.draw_grid(
                &mut tess,
                10.0,
                Color::new(gray_maj, gray_maj, gray_maj, 1.0),
            );
        }

        self.road_renderer.render(
            &self.world.read_resource::<Map>(),
            time.time_seconds,
            &mut tess,
            &self.camera,
            ctx,
            self.world.read_resource::<MapUIState>().map_render_dirty,
        );

        self.instanced_renderer.render(&mut self.world, ctx);

        tess.meshbuilder.build(ctx.gfx).map(|x| x.draw(ctx));

        let mut gui = (*self.world.read_resource::<Gui>()).clone();
        self.gui.render(ctx, &mut self.world, &mut gui);
        *self.world.write_resource::<Gui>() = gui;
    }

    pub fn draw_grid(&mut self, tess: &mut Tesselator, grid_size: f32, color: Color) {
        let screen = tess.screen_box;

        let mut x = (screen.x / grid_size).ceil() * grid_size;
        tess.color = color;
        while x < screen.x + screen.w {
            tess.draw_line(
                Vector2::new(x, screen.y),
                Vector2::new(x, screen.y + screen.h),
                0.01,
            );
            x += grid_size;
        }

        let mut y = (screen.y / grid_size).ceil() * grid_size;
        while y < screen.y + screen.h {
            tess.draw_line(
                Vector2::new(screen.x, y),
                Vector2::new(screen.x + screen.w, y),
                0.01,
            );
            x += grid_size;
            y += grid_size;
        }
    }

    fn manage_timestep(&mut self, delta: f64) {
        let mut time = self.world.write_resource::<TimeInfo>();

        self.time_sync += delta * time.time_speed;
        let diff = self.time_sync - time.time;
        if diff > TIME_STEP * 2.0 {
            self.time_sync = time.time + TIME_STEP;
        }

        if diff > TIME_STEP {
            time.delta = TIME_STEP as f32;
            time.time += TIME_STEP;
            time.time_seconds = time.time as u64;
        } else {
            time.delta = 0.0;
        }
    }

    fn manage_entity_follow(&mut self) {
        if !self
            .world
            .read_resource::<MouseInfo>()
            .just_pressed
            .is_empty()
        {
            self.world.write_resource::<FollowEntity>().0.take();
        }

        if let Some(e) = self.world.read_resource::<FollowEntity>().0 {
            if let Some(pos) = self
                .world
                .read_component::<Transform>()
                .get(e)
                .map(|x| x.position())
            {
                self.camera.camera.position = pos;
            }
        }
    }

    fn manage_io(&mut self, ctx: &Context) {
        *self.world.write_resource::<KeyboardInfo>() = ctx.input.keyboard.clone();
        *self.world.write_resource::<MouseInfo>() = ctx.input.mouse.clone();

        if self.gui.last_kb_captured {
            let kb: &mut KeyboardInfo = &mut self.world.write_resource::<KeyboardInfo>();
            kb.just_pressed.clear();
            kb.is_pressed.clear();
        }

        if self.gui.last_mouse_captured {
            let mouse: &mut MouseInfo = &mut self.world.write_resource::<MouseInfo>();
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

    pub fn unproject(&mut self, pos: Vector2<f32>) -> Vector2<f32> {
        self.camera.unproject_mouse_click(pos)
    }
}
