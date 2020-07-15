use geom::polygon::Polygon;
use mlua::{Lua, UserData, UserDataMethods};

#[derive(Clone)]
pub struct LuaPolygon(pub Polygon);

impl UserData for LuaPolygon {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("extrude", |_, p, (seg, dist): (i32, f32)| {
            p.0.extrude(seg as usize, dist);
            Ok(())
        });
        methods.add_method_mut("split_segment", |_, p, (seg, coeff): (i32, f32)| {
            p.0.split_segment(seg as usize, coeff);
            Ok(())
        });
    }
}

pub fn add_std(lua: &Lua) {
    let poly_rect = lua
        .create_function(|_, (w, h): (f32, f32)| Ok(LuaPolygon(Polygon::rect(w, h))))
        .unwrap();

    lua.globals().set("poly_rect", poly_rect).unwrap();
}
