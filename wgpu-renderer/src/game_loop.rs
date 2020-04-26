use crate::engine::{
    Context, Drawable, FrameContext, GfxContext, InstanceRaw, SpriteBatch, SpriteBatchBuilder,
    Texture,
};
use crate::rendering::imgui_wrapper::ImguiWrapper;
use crate::rendering::CameraHandler;
use cgmath::{Vector2, Vector3};
use scale::engine_interaction::{KeyboardInfo, MouseInfo, RenderStats, TimeInfo};
use scale::gui::Gui;
use scale::interaction::FollowEntity;
use scale::physics::Transform;
use scale::specs::RunNow;
use scale::specs::WorldExt;
use std::time::Instant;
use winit::dpi::PhysicalSize;

pub struct State<'a> {
    camera: CameraHandler,
    sb: SpriteBatch,
    gui: ImguiWrapper,
    world: scale::specs::World,
    dispatcher: scale::specs::Dispatcher<'a, 'a>,
    time_sync: f64,
    last_time: Instant,
}

const TIME_STEP: f64 = 1.0 / 50.0;

impl<'a> State<'a> {
    pub fn new(ctx: &mut Context) -> Self {
        let camera = CameraHandler::new(ctx.gfx.size.0 as f32, ctx.gfx.size.1 as f32, 10.0);

        let tex = Texture::from_path(&ctx.gfx, "resources/car.png").expect("couldn't load car");

        let mut sb = SpriteBatchBuilder::new(tex);

        let mut pos = Transform::new(Vector2::<f32>::new(10.0, 0.0));
        pos.set_angle(0.5);

        sb.instances.push(InstanceRaw::new(
            pos.to_matrix4(),
            Vector3::new(1.0, 1.0, 1.0),
            4.5,
        ));

        let sbb = sb.build(&ctx.gfx);

        let wrapper = ImguiWrapper::new(&mut ctx.gfx);

        let mut world = scale::specs::World::empty();
        let dispatcher = scale::setup(&mut world);

        Self {
            camera,
            sb: sbb,
            gui: wrapper,
            world,
            dispatcher,
            time_sync: 0.0,
            last_time: Instant::now(),
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        let delta = (Instant::now() - self.last_time).as_secs_f64();
        self.last_time = Instant::now();

        self.manage_timestep(delta);

        self.manage_io();

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
        self.sb.draw(ctx);

        let mut gui = (*self.world.read_resource::<Gui>()).clone();
        self.gui.render(ctx, &mut self.world, &mut gui);
        *self.world.write_resource::<Gui>() = gui;
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

    fn manage_io(&mut self) {
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
