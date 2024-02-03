use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
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

    /// hue: [0-360]
    /// saturation: [0-1]
    /// value: [0-1]
    pub fn hsv(hue: f32, sat: f32, val: f32, a: f32) -> Self {
        let c = sat * val;
        let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());

        let (r, g, b) = match hue as i32 / 60 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        let m = val - c;
        Self {
            r: r + m,
            g: g + m,
            b: b + m,
            a,
        }
    }

    pub fn adjust_luminosity(mut self, factor: f32) -> Self {
        self.r = (self.r * factor).min(1.0);
        self.g = (self.g * factor).min(1.0);
        self.b = (self.b * factor).min(1.0);
        self
    }

    pub fn a(self, a: f32) -> Self {
        Self { a, ..self }
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

    pub const PURPLE: Color = Color {
        r: 1.0,
        g: 0.15,
        b: 0.9,
        a: 1.0,
    };
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct LinearColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for LinearColor {
    fn default() -> Self {
        Self::BLACK
    }
}

impl LinearColor {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        LinearColor { r, g, b, a }
    }

    pub fn a(self, v: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: v,
        }
    }

    pub fn gray(level: f32) -> Self {
        Self {
            r: level,
            g: level,
            b: level,
            a: 1.0,
        }
    }

    pub const TRANSPARENT: LinearColor = LinearColor {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub const WHITE: LinearColor = LinearColor {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub const BLACK: LinearColor = LinearColor {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const RED: LinearColor = LinearColor {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const GREEN: LinearColor = LinearColor {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    pub const BLUE: LinearColor = LinearColor {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    pub const CYAN: LinearColor = LinearColor {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub const MAGENTA: LinearColor = LinearColor {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    pub const YELLOW: LinearColor = LinearColor {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    pub const ORANGE: LinearColor = LinearColor {
        r: 1.0,
        g: 0.21,
        b: 0.01,
        a: 1.0,
    };
}

impl Add for LinearColor {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
            a: self.a + rhs.a,
        }
    }
}

pub fn from_srgb(component: f32) -> f32 {
    let a = 0.055;
    if component <= 0.04045 {
        component / 12.92
    } else {
        ((component + a) / (1.0 + a)).powf(2.4)
    }
}

pub fn to_srgb(component: f32) -> f32 {
    let a = 0.055;
    if component <= 0.00031308 {
        component * 12.92
    } else {
        component.powf(0.416666) * (1.0 + a) - a
    }
}

impl From<Color> for LinearColor {
    fn from(color: Color) -> Self {
        LinearColor {
            r: from_srgb(color.r),
            g: from_srgb(color.g),
            b: from_srgb(color.b),
            a: color.a,
        }
    }
}

impl From<LinearColor> for Color {
    fn from(lcolor: LinearColor) -> Self {
        Color {
            r: to_srgb(lcolor.r),
            g: to_srgb(lcolor.g),
            b: to_srgb(lcolor.b),
            a: lcolor.a,
        }
    }
}

impl From<[f32; 4]> for LinearColor {
    fn from(value: [f32; 4]) -> Self {
        Self {
            r: value[0],
            g: value[1],
            b: value[2],
            a: value[3],
        }
    }
}

impl Mul<LinearColor> for f32 {
    type Output = LinearColor;

    fn mul(self, rhs: LinearColor) -> Self::Output {
        LinearColor {
            r: self * rhs.r,
            g: self * rhs.g,
            b: self * rhs.b,
            a: rhs.a,
        }
    }
}

impl From<Color> for [f32; 4] {
    fn from(x: Color) -> [f32; 4] {
        [x.r, x.g, x.b, x.a]
    }
}

impl From<LinearColor> for [f32; 4] {
    fn from(x: LinearColor) -> [f32; 4] {
        [x.r, x.g, x.b, x.a]
    }
}

impl From<&LinearColor> for [f32; 4] {
    fn from(x: &LinearColor) -> [f32; 4] {
        [x.r, x.g, x.b, x.a]
    }
}

impl From<&LinearColor> for LinearColor {
    fn from(x: &LinearColor) -> Self {
        *x
    }
}
