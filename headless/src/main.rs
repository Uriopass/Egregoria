use common::logger::MyLog;
use common::unwrap_or;
use egregoria::engine_interaction::WorldCommands;
use egregoria::Egregoria;
use networking::{Frame, Server, ServerConfiguration, ServerPollResult};
use std::time::{Duration, Instant};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "Egregoria headless", no_version, author = "by Uriopass")]
struct Opt {
    /// Optional server port
    #[structopt(long)]
    port: Option<u16>,

    /// Auto save frequency, in seconds
    #[structopt(long, default_value = "300")]
    autosave: u64,

    /// Always continue running even when everyone is disconnected
    #[structopt(long)]
    always_run: bool,

    /// Timestep in millisecond.
    /// i.e. 20ms = 50FPS
    #[structopt(long, default_value = "20")]
    timestep: u64,
}

fn main() {
    let opt: Opt = Opt::from_args();
    MyLog::init();

    log::info!("starting server with version: {}", goria_version::VERSION);

    let mut w = unwrap_or!(Egregoria::load_from_disk("world"), {
        log::info!("savegame not found defaulting to empty");
        Egregoria::new(10)
    });

    let mut sched = Egregoria::schedule();

    let mut server: Server<Egregoria, WorldCommands> = match Server::start(ServerConfiguration {
        start_frame: Frame(w.get_tick()),
        period: Duration::from_millis(opt.timestep),
        port: opt.port,
        virtual_client: None,
        version: goria_version::VERSION.to_string(),
        always_run: opt.always_run,
    }) {
        Ok(x) => x,
        Err(e) => {
            log::error!("could not start server: {}", e);
            return;
        }
    };
    log::info!("server started!");

    let mut last_saved = Instant::now();

    loop {
        if let ServerPollResult::Input(inputs) = server.poll(&w, Frame(w.get_tick()), None) {
            for frame in inputs {
                assert_eq!(frame.frame.0, w.get_tick() + 1);
                let merged = frame.inputs.into_iter().map(|x| x.inp).collect();
                w.tick(&mut sched, &merged);
            }
        }

        if last_saved.elapsed().as_secs() > opt.autosave {
            w.save_to_disk("world");
            last_saved = Instant::now();
        }

        std::thread::sleep(Duration::from_millis(1));
    }
}
