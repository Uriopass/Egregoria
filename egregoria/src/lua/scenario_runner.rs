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
            mods::eval_f(&l.lock().unwrap(), "paint");
            if mods::call_f(&l.lock().unwrap(), "success").unwrap_or_default() {
                info!("scenario success");
                scenario.l.take();
            }
        }
    }
}

pub fn set_scenario(world: &mut World, name: &str) {
    if let Some(l) = mods::load(name) {
        super::add_world(&l, world);
        mods::eval_f(&l, "init");
        world.write_resource::<RunningScenario>().l = Some(Mutex::new(l));
    }
}
