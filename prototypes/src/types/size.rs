use crate::get_with_err;
use geom::Vec2;
use mlua::{FromLua, Lua, Value};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size2D(Vec2);

impl Size2D {
    pub const ZERO: Self = Self(Vec2::ZERO);
    pub fn new(w: f32, h: f32) -> Self {
        Self(Vec2::new(w, h))
    }

    pub fn w(&self) -> f32 {
        self.0.x
    }

    pub fn h(&self) -> f32 {
        self.0.y
    }
}

impl<'lua> FromLua<'lua> for Size2D {
    fn from_lua(value: Value<'lua>, lua: &'lua Lua) -> mlua::Result<Self> {
        if let Ok(v) = FromLua::from_lua(value.clone(), lua) {
            return Ok(Size2D(v));
        }

        Ok(match value {
            Value::Nil => Size2D(Vec2::ZERO),
            Value::Integer(i) => Size2D(Vec2::new(i as f32, i as f32)),
            Value::Number(n) => Size2D(Vec2::new(n as f32, n as f32)),
            Value::Vector(v) => Size2D(Vec2::new(v.x(), v.y())),
            Value::Table(ref t) => {
                let w = get_with_err::<f32>(t, "w")?;
                let h = get_with_err::<f32>(t, "h")?;
                Size2D(Vec2::new(w, h))
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
