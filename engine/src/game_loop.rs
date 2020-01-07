use crate::components::{Kinematics, MeshRenderComponent, Transform};
use crate::rendering::camera_handler::CameraHandler;
use crate::rendering::render_context::RenderContext;
use crate::resources::{DeltaTime, KeyboardInfo, MouseInfo};
use crate::PHYSICS_UPDATES;

use crate::gui::imgui_wrapper::{Gui, ImGuiWrapper};
use cgmath::InnerSpace;
use ggez::graphics::{Color, Font};
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::input::mouse::MouseButton;
use ggez::{filesystem, graphics, timer, Context, GameResult};
use specs::{Dispatcher, Join, RunNow, World, WorldExt};
use std::collections::HashSet;
use std::iter::FromIterator;

pub struct EngineState<'a, G: Gui> {
    pub world: World,
    pub dispatch: Dispatcher<'a, 'a>,
    pub time: f32,
    pub cam: CameraHandler,
    pub render_enabled: bool,
    pub grid: bool,
    pub font: Font,
    pub imgui_wrapper: ImGuiWrapper,
    _gui: std::marker::PhantomData<G>,
}

impl<'a, G: Gui> EngineState<'a, G> {
    pub(crate) fn new(
        world: World,
        dispatch: Dispatcher<'a, 'a>,
        mut ctx: &mut Context,
    ) -> GameResult<EngineState<'a, G>> {
        println!("{}", filesystem::resources_dir(ctx).display());

        let font = graphics::Font::new(ctx, "/bmonofont-i18n.ttf")?;
        //        let text = graphics::Text::new(("Hello world!", font, 48.0));
        //       let test: Image = graphics::Image::new(ctx, "/test.png")?;

        graphics::set_resizable(ctx, true)?;
        let (width, height) = graphics::size(ctx);
        let imgui_wrapper = ImGuiWrapper::new(&mut ctx);
        Ok(EngineState {
            font,
            world,
            dispatch,
            time: 0.0,
            cam: CameraHandler::new(width, height),
            render_enabled: true,
            grid: true,
            imgui_wrapper,
            _gui: std::marker::PhantomData::default(),
        })
    }
}

impl<'a, G: 'static + Gui> ggez::event::EventHandler for EngineState<'a, G> {
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

        *self.world.write_resource() = DeltaTime(delta / (PHYSICS_UPDATES as f32));
        for _ in 0..PHYSICS_UPDATES {
            self.dispatch.run_now(&self.world);
            self.world.maintain();

            self.world
                .write_resource::<KeyboardInfo>()
                .just_pressed
                .clear();
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
        {
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
        }

        rc.finish()?;

        let gui: G = (&*self.world.read_resource::<G>()).clone();

        // Render game ui
        self.imgui_wrapper.render(ctx, &mut self.world, gui, 1.0);

        graphics::present(ctx)
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
        if y > 0.0 {
            self.cam.easy_camera_movement_keys(ctx, KeyCode::Add);
        }
        if y < 0.0 {
            self.cam.easy_camera_movement_keys(ctx, KeyCode::Subtract);
        }
        self.imgui_wrapper.last_mouse_wheel = y;
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _: KeyMods, _: bool) {
        self.world
            .write_resource::<KeyboardInfo>()
            .just_pressed
            .insert(keycode);
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
