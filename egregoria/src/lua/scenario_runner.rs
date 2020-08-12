use mods::mlua::Lua;
use specs::prelude::*;
use std::sync::Mutex;

#[derive(Default)]
pub struct RunningScenario {
    pub l: Option<Mutex<Lua>>,
}

pub struct RunningScenarioSystem;
impl<'a> System<'a> for RunningScenarioSystem {
    type SystemData = Write<'a, RunningScenario>;

    fn run(&mut self, mut scenario: Self::SystemData) {
        if let Some(l) = &scenario.l {
            let l = l.lock().unwrap();
            mods::eval_f(&l, "Draw");

            let r: Option<bool> = mods::call_f(&l, "Success");
            let is_success = match r {
                Some(x) => x,
                None => {
                    drop(l);

                    scenario.l.take();
                    return;
                }
            };
            if is_success {
                info!("scenario success");
                mods::eval_f(&l, "Cleanup");

                drop(l);
                scenario.l.take();
            }
        }
    }
}

pub fn set_scenario(world: &mut World, name: &str) {
    if let Some(l) = mods::load(name) {
        super::add_egregoria_lua_stdlib(&l, world);
        mods::eval_f(&l, "Init");
        world
            .write_resource::<RunningScenario>()
            .l
            .replace(Mutex::new(l))
            .map(|old| mods::eval_f(&old.lock().unwrap(), "Cleanup"));
    }
}
