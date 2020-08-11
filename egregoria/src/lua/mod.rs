use crate::map_interaction::Itinerary;
use crate::physics::Transform;
use crate::vehicles::{make_vehicle_entity, VehicleComponent, VehicleKind, VehicleState};
use mods::mlua::{Lua, ToLua, UserData, UserDataMethods, Value};
use mods::LuaVec2;
use specs::{Entity, World, WorldExt};

pub mod scenario_runner;

struct LuaWorld {
    w: *mut World,
}

impl UserData for LuaWorld {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("add_car", |_: &Lua, sel: &Self, pos: LuaVec2| unsafe {
            let e = make_vehicle_entity(
                &mut (*sel.w),
                Transform::new(pos.0),
                VehicleComponent {
                    ang_velocity: 0.0,
                    wait_time: 0.0,
                    park_spot: None,
                    state: VehicleState::Driving,
                    kind: VehicleKind::Car,
                },
                Itinerary::none(),
            );
            Ok(LuaEntity(e))
        });

        methods.add_method("pos", |l: &Lua, sel: &Self, e: LuaEntity| unsafe {
            Ok(match (*sel.w).read_storage::<Transform>().get(e.0) {
                Some(t) => LuaVec2(t.position()).to_lua(l).unwrap(),
                None => Value::Nil,
            })
        });
    }
}

#[derive(Clone, Copy)]
struct LuaEntity(Entity);

impl UserData for LuaEntity {}

pub fn add_world(lua: &Lua, w: &mut World) {
    lua.globals()
        .set("world", LuaWorld { w: w as *mut World })
        .unwrap();
}
