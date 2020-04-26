use imgui::{im_str, ColorEdit, EditableColor, Ui};
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use serde::{Deserialize, Serialize};
use specs::World;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    pub const fn gray(level: f32) -> Self {
        Self {
            r: level,
            g: level,
            b: level,
            a: 1.0,
        }
    }

    pub fn from_hex(hex: u64) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as f32 / 255.0,
            g: ((hex >> 8) & 0xFF) as f32 / 255.0,
            b: (hex & 0xFF) as f32 / 255.0,
            a: 1.0,
        }
    }

    pub const TRANSPARENT: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    pub const CYAN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub const MAGENTA: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    pub const YELLOW: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    pub const ORANGE: Color = Color {
        r: 1.0,
        g: 0.5,
        b: 0.1,
        a: 1.0,
    };
}

impl InspectRenderDefault<Color> for Color {
    fn render(_: &[&Color], _: &'static str, _: &mut World, _: &Ui, _: &InspectArgsDefault) {
        unimplemented!()
    }

    fn render_mut(
        data: &mut [&mut Color],
        label: &'static str,
        _: &mut World,
        ui: &Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }

        let c = &mut data[0];
        let mut color_arr = [c.r, c.g, c.b, c.a];
        if ColorEdit::new(&im_str!("{}", label), EditableColor::Float4(&mut color_arr)).build(ui) {
            c.r = color_arr[0];
            c.g = color_arr[1];
            c.b = color_arr[2];
            c.a = color_arr[3];
            true
        } else {
            false
        }
    }
}

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}
