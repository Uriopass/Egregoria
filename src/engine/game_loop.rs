use crate::engine::camera_handler::CameraHandler;
use crate::engine::components::{CircleRender, LineRender, LineToRender, Position, RectRender};
use crate::engine::render_context::RenderContext;
use crate::engine::resources::{DeltaTime, MouseInfo};
use crate::engine::PHYSICS_UPDATES;
use cgmath::Vector2;
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::input::mouse::MouseButton;
use ggez::{filesystem, graphics, timer, Context, GameResult};
use specs::{Dispatcher, Join, RunNow, World, WorldExt};
use std::collections::HashSet;
use std::iter::FromIterator;
use std::time::Instant;

pub struct EngineState<'a> {
    pub world: World,
    pub dispatch: Dispatcher<'a, 'a>,
    pub time: f32,
    pub cam: CameraHandler,
    pub last_time: Instant,
}

impl<'a> EngineState<'a> {
    pub(crate) fn new(
        world: World,
        dispatch: Dispatcher<'a, 'a>,
        ctx: &mut Context,
    ) -> GameResult<EngineState<'a>> {
        println!("{}", filesystem::resources_dir(ctx).display());

        //let font = graphics::Font::new(ctx, "/bmonofont-i18n.ttf")?;
        //let text = graphics::Text::new(("Hello world!", font, 48.0));
        //let test: Image = graphics::Image::new(ctx, "/test.png")?;

        graphics::set_resizable(ctx, true)?;
        Ok(EngineState {
            world,
            dispatch,
            time: 0.,
            cam: CameraHandler::new(),
            last_time: Instant::now(),
        })
    }
}

impl<'a> ggez::event::EventHandler for EngineState<'a> {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let delta = timer::delta(ctx).as_secs_f32();
        self.time += delta;
        *self.world.write_resource() = MouseInfo {
            unprojected: self.cam.unproject_mouse_click(ctx),
            buttons: HashSet::from_iter(
                vec![MouseButton::Left, MouseButton::Right, MouseButton::Middle]
                    .into_iter()
                    .filter(|x| ggez::input::mouse::button_pressed(ctx, *x)),
            ),
        };

        *self.world.write_resource() = DeltaTime(delta / (PHYSICS_UPDATES as f32));

        for _ in 0..PHYSICS_UPDATES {
            self.dispatch.run_now(&self.world);
            self.world.maintain();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.cam.easy_camera_movement(ctx);
        self.cam.update(ctx);

        let mut rc = RenderContext::new(&mut self.cam, ctx);
        rc.clear();

        let positions = self.world.read_component::<Position>();
        let circle_render = self.world.read_component::<CircleRender>();
        let rect_render = self.world.read_component::<RectRender>();
        let line_to_render = self.world.read_component::<LineToRender>();
        let line_render = self.world.read_component::<LineRender>();

        for (pos, lr) in (&positions, &line_to_render).join() {
            let ppos = pos.0;
            let e = lr.to;
            let pos2: Vector2<f32> = positions.get(e).unwrap().0;
            rc.sr.draw_line(ppos, pos2);
        }

        for lr in (&line_render).join() {
            let start = lr.start;
            let end = lr.end;
            rc.sr.color = lr.color;
            rc.sr.draw_line(start, end);
        }

        for (pos, rr) in (&positions, &rect_render).join() {
            rc.sr.color = rr.color;
            rc.sr.draw_rect(
                pos.0 - Vector2::new(rr.width / 2., rr.height / 2.),
                rr.width,
                rr.height,
            )
        }

        for (pos, cr) in (&positions, &circle_render).join() {
            let pos = pos.0;
            rc.sr.color = cr.color;
            rc.sr.draw_circle(pos, cr.radius);
        }

        rc.finish()?;
        graphics::present(ctx)
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
        if y > 0. {
            self.cam.easy_camera_movement_keys(ctx, KeyCode::Add);
        }
        if y < 0. {
            self.cam.easy_camera_movement_keys(ctx, KeyCode::Subtract);
        }
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _: KeyMods, _: bool) {
        self.cam.easy_camera_movement_keys(ctx, keycode);
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.cam.resize(ctx, width, height);
    }
}
