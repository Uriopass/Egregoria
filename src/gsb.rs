use crate::camera::Camera;
use ggez::Context;
use ggez::graphics;
use ggez::nalgebra::Vector2;
use ggez::input;
use ggez::input::mouse::MouseButton;
use ggez::input::keyboard::KeyCode;

#[allow(dead_code)]
pub struct GSB {
    camera: Camera,
    last_pos: Vector2<f32>,
}

#[allow(dead_code)]
impl GSB {
    pub fn new() -> GSB {
        GSB {
            camera: Camera::new(800., 600.),
            last_pos: Vector2::new(0., 0.),
        }
    }

    pub fn center_camera(&mut self, ctx: &mut Context) {
        self.camera.position.x = 0.;
        self.camera.position.y = 0.;
        self.update(ctx);
    }

    pub fn update(&mut self, ctx: &mut Context) {
        self.camera.update();
        graphics::set_projection(ctx, self.camera.projection);
    }

    /*
    pub fn get_screen_box(&self) -> Rectangle {
        let mut screen: Rectangle = Rectangle::new();
        let mut width: f32 = GSB.camera.viewportWidth * GSB.camera.zoom;
        let mut height: f32 = GSB.camera.viewportHeight * GSB.camera.zoom;
        screen.x = GSB.camera.position.x - width / 2.0;
        screen.y = GSB.camera.position.y - height / 2.0;
        screen.width = width;
        screen.height = height;
        return screen;
    }
    */

    pub fn resize(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.camera.set_viewport(width, height);
        self.update(ctx);
    }

    pub fn unproject_mouse_click(&self, ctx: &Context) -> Vector2<f32> {
        let haha = ggez::input::mouse::position(ctx);
        self.camera.unproject([haha.x, haha.y].into())
    }

    pub fn clear(&self, ctx: &mut Context) {
        graphics::clear(ctx, graphics::Color::from_rgb(0, 0, 0));
        graphics::set_window_title(ctx, format!("{} FPS", ggez::timer::fps(ctx)).as_ref());
    }

    pub fn easy_camera_movement(&mut self, ctx: &mut Context) {
        let p = self.unproject_mouse_click(ctx);
        if input::mouse::button_pressed(ctx, MouseButton::Right) {
            self.camera.position -= p - self.last_pos;
            self.update(ctx);
        }
        self.last_pos = self.unproject_mouse_click(ctx);
    }

    pub fn easy_camera_movement_keys(&mut self, ctx: &mut Context, keycode: KeyCode) {
        if keycode == KeyCode::Add {
            self.camera.zoom *= 1.2;
            self.update(ctx);
        }
        if keycode == KeyCode::Subtract {
            self.camera.zoom /= 1.2;
            self.update(ctx);
        }
    }
}
