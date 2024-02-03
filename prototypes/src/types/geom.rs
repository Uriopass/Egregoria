use geom::{vec2, vec3, Color, Vec2, Vec3};
use mlua::{FromLua, Value};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct LuaVec2(pub Vec2);

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct LuaVec3(pub Vec3);

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct LuaColor(pub Color);

impl Into<Vec2> for LuaVec2 {
    fn into(self) -> Vec2 {
        self.0
    }
}

impl Into<Vec3> for LuaVec3 {
    fn into(self) -> Vec3 {
        self.0
    }
}

impl From<Vec3> for LuaVec3 {
    fn from(v: Vec3) -> Self {
        Self(v)
    }
}

impl From<Vec2> for LuaVec2 {
    fn from(v: Vec2) -> Self {
        Self(v)
    }
}

impl From<Color> for LuaColor {
    fn from(c: Color) -> Self {
        Self(c)
    }
}

impl From<LuaColor> for Color {
    fn from(c: LuaColor) -> Self {
        c.0
    }
}

impl<'a> FromLua<'a> for LuaColor {
    fn from_lua(value: Value<'a>, _: &'a mlua::Lua) -> mlua::Result<Self> {
        let t = match value {
            Value::Vector(v) => {
                return Ok(Self(Color::new(v.x(), v.y(), v.z(), 1.0)));
            }
            Value::Table(t) => t,
            _ => {
                return Err(mlua::Error::FromLuaConversionError {
                    from: value.type_name(),
                    to: "Color",
                    message: Some("expected a table or vector".to_string()),
                })
            }
        };
        if let Ok(r) = t.get(1) {
            return Ok(Self(Color::new(
                r,
                t.get(2)?,
                t.get(3)?,
                t.get(4).unwrap_or(1.0),
            )));
        }

        let r = t.get("r")?;
        let g = t.get("g")?;
        let b = t.get("b")?;
        let a = t.get("a").unwrap_or(1.0);
        Ok(Self(Color::new(r, g, b, a)))
    }
}

impl<'a> FromLua<'a> for LuaVec2 {
    fn from_lua(value: Value<'a>, _: &'a mlua::Lua) -> mlua::Result<Self> {
        let t = match value {
            Value::Vector(v) => return Ok(Self(vec2(v.x(), v.y()))),
            Value::Table(t) => t,
            _ => {
                return Err(mlua::Error::FromLuaConversionError {
                    from: value.type_name(),
                    to: "Vec2",
                    message: Some("expected a table or vector".to_string()),
                })
            }
        };
        if let Ok(x) = t.get(1) {
            return Ok(Self(vec2(x, t.get(2)?)));
        }

        let x = t.get("x")?;
        let y = t.get("y")?;
        Ok(Self(vec2(x, y)))
    }
}

impl<'a> FromLua<'a> for LuaVec3 {
    fn from_lua(value: Value<'a>, _: &'a mlua::Lua) -> mlua::Result<Self> {
        let t = match value {
            Value::Vector(v) => {
                return Ok(Self(vec3(v.x(), v.y(), v.z())));
            }
            Value::Table(t) => t,
            _ => {
                return Err(mlua::Error::FromLuaConversionError {
                    from: value.type_name(),
                    to: "Vec3",
                    message: Some("expected a table or vector".to_string()),
                })
            }
        };
        if let Ok(x) = t.get(1) {
            return Ok(Self(vec3(x, t.get(2)?, t.get(3)?)));
        }

        let x = t.get("x")?;
        let y = t.get("y")?;
        let z = t.get("z")?;
        Ok(Self(vec3(x, y, z)))
    }
}
