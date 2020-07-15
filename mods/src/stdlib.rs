use geom::polygon::Polygon;
use geom::Vec2;
use mlua::prelude::LuaResult;
use mlua::{Lua, MetaMethod, UserData, UserDataMethods};

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
        methods.add_method_mut("barycenter", |_, p, (): ()| Ok(LuaVec2(p.0.barycenter())));
        methods.add_method_mut("translate", |_, p, vec: LuaVec2| {
            p.0.translate(vec.0);
            Ok(())
        });
        methods.add_method_mut("rotate", |_, p, cossin: LuaVec2| {
            p.0.rotate(cossin.0);
            Ok(())
        });
    }
}

#[derive(Clone, Copy)]
pub struct LuaVec2(pub Vec2);

impl UserData for LuaVec2 {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("magnitude", |_, vec, ()| Ok(vec.0.magnitude()));

        methods.add_meta_function(MetaMethod::Unm, |_, vec1: LuaVec2| Ok(LuaVec2(-vec1.0)));

        methods.add_meta_function(MetaMethod::Add, |_, (vec1, vec2): (LuaVec2, LuaVec2)| {
            Ok(LuaVec2(vec1.0 + vec2.0))
        });

        methods.add_meta_function(MetaMethod::Sub, |_, (vec1, vec2): (LuaVec2, LuaVec2)| {
            Ok(LuaVec2(vec1.0 - vec2.0))
        });

        methods.add_meta_function(MetaMethod::Mul, |_, (vec1, scalar): (LuaVec2, f32)| {
            Ok(LuaVec2(vec1.0 * scalar))
        });

        methods.add_meta_function(MetaMethod::Mul, |_, (scalar, vec1): (f32, LuaVec2)| {
            Ok(LuaVec2(scalar * vec1.0))
        });

        methods.add_meta_function(MetaMethod::Mul, |_, (vec1, vec2): (LuaVec2, LuaVec2)| {
            Ok(LuaVec2(vec1.0 * vec2.0))
        });
    }
}

macro_rules! std_add_fn {
    ($l: expr, $e: expr) => {
        $l.globals()
            .set(stringify!($e), $l.create_function($e).unwrap())
            .unwrap();
    };
}

fn poly_rect(_: &Lua, (w, h): (f32, f32)) -> LuaResult<LuaPolygon> {
    Ok(LuaPolygon(Polygon::rect(w, h)))
}

fn rand_in(_: &Lua, (min, max): (f32, f32)) -> LuaResult<f32> {
    Ok(min + rand::random::<f32>() * (max - min))
}

pub fn add_std(lua: &Lua) {
    std_add_fn!(lua, poly_rect);
    std_add_fn!(lua, rand_in);
}
