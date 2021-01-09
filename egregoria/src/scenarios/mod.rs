use crate::map_dynamic::Itinerary;
use crate::rendering::immediate::ImmediateDraw;
use crate::vehicles::{make_vehicle_entity, Vehicle, VehicleKind, VehicleState};
use crate::{Egregoria, ParCommandBuffer};
use geom::Color;
use geom::Transform;
use geom::Vec2;
use legion::{Entity, Resources};
use mods::mlua::{Lua, ToLua, UserData, UserDataMethods, Value};
use mods::LuaVec2;

pub mod scenario_runner;

struct LuaWorld {
    w: *mut Egregoria,
}

unsafe impl Send for LuaWorld {}

impl UserData for LuaWorld {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method(
            "add_car",
            |_: &Lua, sel: &Self, (pos, dir, objective): (LuaVec2, LuaVec2, LuaVec2)| unsafe {
                let e = make_vehicle_entity(
                    &mut (*sel.w),
                    Transform::new_cos_sin(pos.0, dir.0.try_normalize().unwrap_or(Vec2::UNIT_X)),
                    Vehicle {
                        ang_velocity: 0.0,
                        wait_time: 0.0,
                        state: VehicleState::Driving,
                        kind: VehicleKind::Car,
                        flag: 0,
                    },
                    Itinerary::simple(vec![objective.0]),
                    true,
                );
                Ok(LuaEntity(e))
            },
        );

        methods.add_method("pos", |l: &Lua, sel: &Self, e: LuaEntity| unsafe {
            Ok(match (*sel.w).comp::<Transform>(e.0) {
                Some(t) => LuaVec2(t.position()).to_lua(l).unwrap(),
                None => Value::Nil,
            })
        });

        methods.add_method("remove", |_: &Lua, sel: &Self, e: LuaEntity| unsafe {
            (*sel.w).write::<ParCommandBuffer>().kill(e.0);
            Ok(())
        });
    }
}

struct LuaDraw {
    w: *mut Resources,
    col: Color,
}

unsafe impl Send for LuaDraw {}

impl UserData for LuaDraw {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("circle", |_, sel, (pos, size): (LuaVec2, f32)| unsafe {
            (*sel.w)
                .get_mut::<ImmediateDraw>()
                .unwrap()
                .circle(pos.0, size)
                .color(sel.col);
            Ok(())
        });
        methods.add_method_mut("color", |_, sel, col: LuaColor| {
            sel.col = col.0;
            Ok(())
        });
    }
}

#[derive(Copy, Clone)]
struct LuaColor(Color);
impl UserData for LuaColor {}

#[derive(Copy, Clone)]
struct LuaEntity(Entity);

impl UserData for LuaEntity {}

fn color(_: &Lua, (r, g, b, a): (f32, f32, f32, f32)) -> mods::mlua::Result<LuaColor> {
    Ok(LuaColor(Color { r, g, b, a }))
}

pub fn add_egregoria_lua_stdlib(lua: &Lua, w: &mut Egregoria) {
    lua.globals()
        .set(
            "world",
            LuaWorld {
                w: w as *mut Egregoria,
            },
        )
        .unwrap();
    lua.globals()
        .set(
            "draw",
            LuaDraw {
                w: &mut w.resources as *mut Resources,
                col: Color::WHITE,
            },
        )
        .unwrap();
    mods::add_fn(lua, "color", color)
}
