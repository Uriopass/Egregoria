use crate::engine::components::{Kinematics, MeshRenderComponent, Transform};
use crate::engine::rendering::camera_handler::CameraHandler;
use crate::engine::rendering::render_context::RenderContext;
use crate::engine::resources::{DeltaTime, MouseInfo};
use crate::engine::PHYSICS_UPDATES;

use crate::cars::car_data::CarComponent;
use crate::cars::car_data::CarObjective::Terminal;
use cgmath::num_traits::Pow;
use cgmath::{InnerSpace, Vector2, Zero};
use ggez::graphics::{Color, Font, Text, TextFragment};
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::input::mouse::MouseButton;
use ggez::{filesystem, graphics, timer, Context, GameResult};
use specs::{Dispatcher, Join, RunNow, World, WorldExt};
use std::collections::HashSet;
use std::iter::FromIterator;
use std::ops::Mul;

pub struct EngineState<'a> {
    pub world: World,
    pub dispatch: Dispatcher<'a, 'a>,
    pub time: f32,
    pub cam: CameraHandler,
    pub render_enabled: bool,
    pub grid: bool,
    pub font: Font,
}

impl<'a> EngineState<'a> {
    pub(crate) fn new(
        world: World,
        dispatch: Dispatcher<'a, 'a>,
        ctx: &mut Context,
    ) -> GameResult<EngineState<'a>> {
        println!("{}", filesystem::resources_dir(ctx).display());

        let font = graphics::Font::new(ctx, "/bmonofont-i18n.ttf")?;
        //        let text = graphics::Text::new(("Hello world!", font, 48.0));
        //       let test: Image = graphics::Image::new(ctx, "/test.png")?;

        graphics::set_resizable(ctx, true)?;
        Ok(EngineState {
            font,
            world,
            dispatch,
            time: 0.0,
            cam: CameraHandler::new(),
            render_enabled: true,
            grid: true,
        })
    }
}

impl<'a> ggez::event::EventHandler for EngineState<'a> {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let delta = timer::delta(ctx).as_secs_f32().min(1.0 / 100.0);
        self.time += delta;
        *self.world.write_resource() = MouseInfo {
            unprojected: self.cam.unproject_mouse_click(ctx),
            buttons: HashSet::from_iter(
                vec![MouseButton::Left, MouseButton::Right, MouseButton::Middle]
                    .into_iter()
                    .filter(|x| ggez::input::mouse::button_pressed(ctx, *x)),
            ),
        };

        {
            let transforms = self.world.read_component::<Transform>();
            let mut cars = self.world.write_component::<CarComponent>();
            for (mut car, trans) in (&mut cars, &transforms).join() {
                //car.objective = Some(trans.get_position() + car.direction + car.normal());
                car.objective = Terminal(self.cam.unproject_mouse_click(ctx));
            }
        }

        *self.world.write_resource() = DeltaTime(delta / (PHYSICS_UPDATES as f32));

        for _ in 0..PHYSICS_UPDATES {
            self.dispatch.run_now(&self.world);
            self.world.maintain();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.cam
            .easy_camera_movement(ctx, timer::delta(ctx).as_secs_f32());
        self.cam.update(ctx);

        let mut rc = RenderContext::new(&mut self.cam, ctx, self.font);
        rc.clear();

        if self.grid && rc.cam.camera.zoom > 3.0 {
            let gray_maj = (rc.cam.camera.zoom / 40.0).min(0.2);
            let gray_min = gray_maj / 2.0;
            if rc.cam.camera.zoom > 6.0 {
                rc.draw_grid(1.0, Color::new(gray_min, gray_min, gray_min, 1.0));
            }
            rc.draw_grid(10.0, Color::new(gray_maj, gray_maj, gray_maj, 1.0));
            rc.flush()?;
        }

        let transforms = self.world.read_component::<Transform>();
        let kinematics = self.world.read_component::<Kinematics>();
        let mesh_render = self.world.read_component::<MeshRenderComponent>();

        if self.render_enabled {
            for (trans, mr) in (&transforms, &mesh_render).join() {
                for order in &mr.orders {
                    order.draw(trans, &transforms, &mut rc);
                }
            }
        }

        rc.flush()?;

        for (trans, kin) in (&transforms, &kinematics).join() {
            let v = kin.velocity.magnitude();
            let pos = trans.get_position();
            rc.draw_text(
                &format!("{:.2} m/s", v),
                pos,
                0.5,
                Color::new(0.0, 0.0, 1.0, 1.0),
            )?;
        }

        rc.finish()?;
        graphics::present(ctx)
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
        if y > 0.0 {
            self.cam.easy_camera_movement_keys(ctx, KeyCode::Add);
        }
        if y < 0.0 {
            self.cam.easy_camera_movement_keys(ctx, KeyCode::Subtract);
        }
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _: KeyMods, _: bool) {
        if keycode == KeyCode::R {
            self.render_enabled = !self.render_enabled;
        }
        if keycode == KeyCode::G {
            self.grid = !self.grid;
        }
        self.cam.easy_camera_movement_keys(ctx, keycode);
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.cam.resize(ctx, width, height);
    }
}
