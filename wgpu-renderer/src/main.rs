use log::LevelFilter;
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
        .filter(Some("gfx_backend_vulkan"), LevelFilter::Warn)
        .format(move |f, r| {
            let t = Instant::now().duration_since(start).as_micros();
            let mp = r.module_path_static();
            let v = mp.and_then(|x| x.split(':').last());
            writeln!(
                f,
                "[{:9} {:5} {:15}] {}",
                t,
                r.metadata().level().to_string(),
                v.unwrap_or_default(),
                r.args()
            )
        })
        .init();

    let mut ctx = engine::Context::new();

    let state = game_loop::State::new(&mut ctx);
    ctx.start(state);
}
