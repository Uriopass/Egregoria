use crate::map_dynamic::Itinerary;
use crate::physics::Transform;
use crate::rendering::immediate::ImmediateDraw;
use crate::rendering::Color;
use crate::utils::delete_entity;
use crate::vehicles::{make_vehicle_entity, VehicleComponent, VehicleKind, VehicleState};
use geom::Vec2;
use mods::mlua::{Lua, ToLua, UserData, UserDataMethods, Value};
use mods::LuaVec2;
use specs::{Entity, World, WorldExt};

pub mod scenario_runner;

struct LuaWorld {
    w: *mut World,
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
                    VehicleComponent {
                        ang_velocity: 0.0,
                        wait_time: 0.0,
                        park_spot: None,
                        state: VehicleState::Driving,
                        kind: VehicleKind::Car,
                    },
                    Itinerary::simple(vec![objective.0]),
                    true,
                );
                Ok(LuaEntity(e))
            },
        );

        methods.add_method("pos", |l: &Lua, sel: &Self, e: LuaEntity| unsafe {
            Ok(match (*sel.w).read_storage::<Transform>().get(e.0) {
                Some(t) => LuaVec2(t.position()).to_lua(l).unwrap(),
                None => Value::Nil,
            })
        });

        methods.add_method("remove", |_: &Lua, sel: &Self, e: LuaEntity| unsafe {
            delete_entity(&mut (*sel.w), e.0);
            Ok(())
        });
    }
}

struct LuaDraw {
    w: *mut World,
    col: Color,
}

unsafe impl Send for LuaDraw {}

impl UserData for LuaDraw {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("circle", |_, sel, (pos, size): (LuaVec2, f32)| unsafe {
            (*sel.w)
                .write_resource::<ImmediateDraw>()
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

#[derive(Clone, Copy)]
struct LuaColor(Color);
impl UserData for LuaColor {}

#[derive(Clone, Copy)]
struct LuaEntity(Entity);

impl UserData for LuaEntity {}

fn color(_: &Lua, (r, g, b, a): (f32, f32, f32, f32)) -> mods::mlua::Result<LuaColor> {
    Ok(LuaColor(Color { r, g, b, a }))
}

pub fn add_egregoria_lua_stdlib(lua: &Lua, w: &mut World) {
    lua.globals()
        .set("world", LuaWorld { w: w as *mut World })
        .unwrap();
    lua.globals()
        .set(
            "draw",
            LuaDraw {
                w: w as *mut World,
                col: Color::WHITE,
            },
        )
        .unwrap();
    mods::add_fn(lua, "color", color)
}
