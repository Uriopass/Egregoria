use crate::{get_lua, LuaVec2};
use mlua::{FromLua, Lua, Value};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size2D {
    pub w: f32,
    pub h: f32,
}

impl Size2D {
    pub const ZERO: Self = Self { w: 0.0, h: 0.0 };

    #[inline]
    pub fn new(w: f32, h: f32) -> Self {
        Self { w, h }
    }

    #[inline]
    pub fn area(&self) -> f32 {
        self.w * self.h
    }

    #[inline]
    pub fn diag(&self) -> f32 {
        self.w.hypot(self.h)
    }
}

impl<'lua> FromLua<'lua> for Size2D {
    fn from_lua(value: Value<'lua>, lua: &'lua Lua) -> mlua::Result<Self> {
        if let Ok(v) = <LuaVec2 as FromLua>::from_lua(value.clone(), lua) {
            return Ok(Self { w: v.0.x, h: v.0.y });
        }

        Ok(match value {
            Value::Nil => Size2D::ZERO,
            Value::Integer(i) => Size2D::new(i as f32, i as f32),
            Value::Number(n) => Size2D::new(n as f32, n as f32),
            Value::Vector(v) => Size2D::new(v.x(), v.y()),
            Value::Table(ref t) => {
                let w = get_lua::<f32>(t, "w")?;
                let h = get_lua::<f32>(t, "h")?;
                Size2D::new(w, h)
            }
            _ => {
                return Err(mlua::Error::FromLuaConversionError {
                    from: value.type_name(),
                    to: "Size",
                    message: Some("expected number, table or vector".to_string()),
                })
            }
        })
    }
}
