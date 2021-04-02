use common::logger::MyLog;
use common::timestep::UP_DT;
use common::unwrap_or;
use egregoria::engine_interaction::WorldCommands;
use egregoria::{Egregoria, SerPreparedEgregoria};
use networking::{Frame, Server, ServerConfiguration, ServerPollResult};
use structopt::StructOpt;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Server port
    #[structopt(long)]
    port: Option<u16>,
}

fn main() {
    let opt = Opt::from_args();
    MyLog::init();

    log::info!("starting server...");

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
        }) {
            Ok(x) => x,
            Err(e) => {
                log::error!("could not start server: {}", e);
                return;
            }
        };
    log::info!("server started!");

    loop {
        if let ServerPollResult::Input(inputs) = server.poll(
            &|| (SerPreparedEgregoria::from(&w), Frame(w.get_tick())),
            None,
        ) {
            for frame in inputs {
                assert_eq!(frame.frame.0, w.get_tick() + 1);
                for input in frame.inputs {
                    w.tick(UP_DT.as_secs_f64(), &mut sched, &input.inp);
                }
            }
        }
    }
}
