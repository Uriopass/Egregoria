use mods::mlua::Lua;
use specs::prelude::*;

#[derive(Default)]
pub struct RunningScenario {
    pub l: Option<Lua>,
}

unsafe impl Send for RunningScenario {}
unsafe impl Sync for RunningScenario {}

pub struct RunningScenarioSystem;
impl<'a> System<'a> for RunningScenarioSystem {
    type SystemData = Write<'a, RunningScenario>;

    fn run(&mut self, mut scenario: Self::SystemData) {
        if let Some(l) = &scenario.l {
            if mods::call_f(l, "success").unwrap_or_default() {
                info!("scenario success");
                scenario.l.take();
            }
        }
    }
}

pub fn set_scenario(world: &mut World, name: &str) {
    if let Some(l) = mods::load(name) {
        super::set_state(&l, world);
        mods::eval_f(&l, "init");
        world.write_resource::<RunningScenario>().l = Some(l);
    }
}
