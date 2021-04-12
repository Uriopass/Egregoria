use common::logger::MyLog;
use common::timestep::UP_DT;
use common::unwrap_or;
use egregoria::engine_interaction::WorldCommands;
use egregoria::{Egregoria, SerPreparedEgregoria};
use networking::{Frame, Server, ServerConfiguration, ServerPollResult};
use std::convert::TryFrom;
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
}

fn main() {
    let opt: Opt = Opt::from_args();
    MyLog::init();

    log::info!("starting server with version: {}", goria_version::VERSION);

    let mut w = unwrap_or!(Egregoria::load_from_disk("world"), {
        log::info!("savegame not found defaulting to empty");
        Egregoria::empty()
    });

    let mut sched = Egregoria::schedule();

    let mut server: Server<SerPreparedEgregoria, WorldCommands> =
        match Server::start(ServerConfiguration {
            start_frame: Frame(w.get_tick()),
            period: UP_DT,
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
        if let ServerPollResult::Input(inputs) = server.poll(
            &|| {
                (
                    SerPreparedEgregoria::try_from(&w).expect("could not serialize server"),
                    Frame(w.get_tick()),
                )
            },
            None,
        ) {
            for frame in inputs {
                assert_eq!(frame.frame.0, w.get_tick() + 1);
                for input in frame.inputs {
                    w.tick(&mut sched, &input.inp);
                }
            }
        }

        if last_saved.elapsed().as_secs() > opt.autosave {
            w.save_to_disk("world");
            last_saved = Instant::now();
        }

        std::thread::sleep(Duration::from_millis(1));
    }
}
