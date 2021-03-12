use geom::Polygon;
use geom::Vec2;
use mlua::prelude::LuaResult;
use mlua::{FromLuaMulti, Lua, MetaMethod, ToLuaMulti, UserData, UserDataMethods};

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

#[derive(Copy, Clone)]
pub struct LuaVec2(pub Vec2);

impl UserData for LuaVec2 {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("magnitude", |_, vec, ()| Ok(vec.0.magnitude()));
        methods.add_method("normalize", |_, vec, ()| {
            Ok(LuaVec2(vec.0.try_normalize().unwrap_or(Vec2::UNIT_X)))
        });
        methods.add_method("distance", |_, vec, v: LuaVec2| Ok(vec.0.distance(v.0)));
        methods.add_method("x", |_, vec, ()| Ok(vec.0.x));
        methods.add_method("y", |_, vec, ()| Ok(vec.0.y));

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

        methods.add_meta_function(MetaMethod::ToString, |_, vec: LuaVec2| {
            Ok(format!("({:.4}, {:.4})", vec.0.x, vec.0.y))
        });
    }
}

pub fn add_fn<'lua, 'callback, A, R, F>(l: &'lua Lua, name: &str, f: F)
where
    'lua: 'callback,
    A: FromLuaMulti<'callback>,
    R: ToLuaMulti<'callback>,
    F: 'static + Send + Fn(&'callback Lua, A) -> mlua::Result<R>,
{
    l.globals()
        .set(name, l.create_function(f).unwrap())
        .unwrap()
}

fn poly_rect(_: &Lua, (w, h): (f32, f32)) -> LuaResult<LuaPolygon> {
    Ok(LuaPolygon(Polygon::rect(w, h)))
}

fn vec2(_: &Lua, (x, y): (f32, f32)) -> LuaResult<LuaVec2> {
    Ok(LuaVec2(Vec2 { x, y }))
}

pub fn add_std(lua: &Lua) {
    add_fn(lua, "poly_rect", poly_rect);
    add_fn(lua, "vec2", vec2);
}
