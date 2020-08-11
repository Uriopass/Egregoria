use argh::FromArgs;
use egregoria::engine_interaction::TimeInfo;
use egregoria::specs::WorldExt;
use egregoria::EgregoriaState;
use log::LevelFilter;

#[derive(FromArgs)]
#[argh(description = "\
Egregoria's headless cli for running egregoria scenarios.\n\
Example: goria test.lua")]
struct Args {
    #[argh(positional)]
    scenario: Vec<String>,
}

fn main() {
    env_logger::builder().filter(None, LevelFilter::Info).init();

    let args: Args = argh::from_env();

    for scenario in args.scenario {
        let state = egregoria::EgregoriaState::setup();
        run(state, &scenario)
    }
}

fn run(mut state: EgregoriaState, name: &str) {
    let l = match mods::load(name) {
        Some(l) => l,
        None => {
            return;
        }
    };

    egregoria::lua::add_world(&l, &mut state.world);
    mods::eval_f(&l, "init");

    for i in 1..1000 {
        step(&mut state);

        let v: Option<bool> = mods::call_f(&l, "success");
        let v = match v {
            Some(x) => x,
            None => {
                return;
            }
        };

        if v {
            log::info!("success for {} at iteration {}", name, i);
            return;
        }
    }

    log::warn!("failure for {}", name);
}

const TIME_STEP: f64 = 1.0 / 30.0;

fn step(state: &mut EgregoriaState) {
    {
        let mut time = state.world.write_resource::<TimeInfo>();
        time.delta = TIME_STEP as f32;
        time.time_speed = 1.0;
        time.time += TIME_STEP;
        time.time_seconds = time.time as u64;
    }
    state.run();
}
