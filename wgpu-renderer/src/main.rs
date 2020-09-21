use log::{Level, LevelFilter};
use std::io::Write;
use std::time::Instant;

mod debug;
mod engine;
mod game_loop;
mod geometry;
mod rendering;

fn main() {
    let start = Instant::now();
    env_logger::builder()
        .filter(None, LevelFilter::Info)
        .filter(Some("wgpu_core"), LevelFilter::Warn)
        .filter(Some("gfx_memory"), LevelFilter::Off)
        .filter(Some("gfx_backend_vulkan"), LevelFilter::Warn)
        .format(move |f, r| {
            let time = Instant::now().duration_since(start).as_micros();
            if r.level() > Level::Warn {
                let module_path = r
                    .module_path_static()
                    .and_then(|x| x.split(':').last())
                    .unwrap_or_default();
                writeln!(
                    f,
                    "[{:9} {:5} {:12}] {}",
                    time,
                    r.metadata().level().to_string(),
                    module_path,
                    r.args()
                )
            } else {
                writeln!(
                    f,
                    "[{:9} {:5} {}:{}] {}",
                    time,
                    r.metadata().level().to_string(),
                    r.file().unwrap_or_default(),
                    r.line().unwrap_or_default(),
                    r.args()
                )
            }
        })
        .init();

    let mut ctx = engine::Context::new();

    let state = game_loop::State::new(&mut ctx);
    ctx.start(state);
}
