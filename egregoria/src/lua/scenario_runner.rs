use crate::Egregoria;
use legion::system;
use mods::mlua::Lua;
use std::sync::Mutex;

#[derive(Default)]
pub struct RunningScenario {
    pub l: Option<Mutex<Lua>>,
}

#[system]
pub fn run_scenario(#[resource] scenario: &mut RunningScenario) {
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

pub fn set_scenario(goria: &mut Egregoria, name: &str) {
    if let Some(l) = mods::load(name) {
        super::add_egregoria_lua_stdlib(&l, goria);
        mods::eval_f(&l, "Init");
        goria
            .write_resource::<RunningScenario>()
            .l
            .replace(Mutex::new(l))
            .map(|old| mods::eval_f(&old.lock().unwrap(), "Cleanup"));
    }
}
