use log::LevelFilter;
use networking::{
    Client, ConnectConf, Frame, PollResult, Server, ServerConfiguration, ServerPollResult,
};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use std::time::Duration;
use Action::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
struct World {
    incr_a: u64,
    incr_b: u64,
    tick: u64,
    pad: Vec<u8>,
}

impl World {
    fn apply(&mut self, acts: Vec<Action>) {
        self.tick += 1;
        for a in acts {
            match a {
                IncrA => self.incr_a += 1,
                IncrB => self.incr_b += 1,
                DoNothing => {}
            }
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, Default)]
enum Action {
    #[default]
    DoNothing,
    IncrA,
    IncrB,
}

const UP_DT: Duration = Duration::from_millis(50);

pub fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    serv_c();

    std::thread::sleep(Duration::from_millis(1000));

    let mut client2: Client<World, Action> = Client::connect(ConnectConf {
        name: "client".into(),
        addr: Ipv4Addr::LOCALHOST.into(),
        port: None,
        frame_buffer_advance: 10,
        version: "v1".to_string(),
    })
    .unwrap();

    let mut world: World;
    loop {
        if let PollResult::GameWorld(_, w) = client2.poll(DoNothing) {
            log::info!("client got world from server: length is {:?}", w.pad.len());
            world = w;
            break;
        }
        std::thread::sleep(Duration::from_millis(100))
    }

    for _ in 1usize.. {
        let res = client2.poll(IncrA);
        if let PollResult::Input(inp) = res {
            for inp in inp {
                log::info!(
                    "client got input from server: {:?} w is now {} {} {}",
                    inp,
                    world.tick,
                    world.incr_a,
                    world.incr_b,
                );
                assert_eq!(world.tick + 1, inp.frame.0);
                world.apply(inp.inputs.into_iter().map(|x| x.inp).collect());
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

pub fn serv_c() {
    let mut world = World {
        incr_a: 0,
        incr_b: 0,
        tick: 5,
        pad: vec![0; 100000],
    };

    let mut serv = Server::start(ServerConfiguration {
        start_frame: Frame(world.tick),
        period: UP_DT,
        port: None,
        virtual_client: None,
        version: "v1".to_string(),
        always_run: true,
    })
    .unwrap();

    std::thread::spawn(move || loop {
        if let ServerPollResult::Input(acts) = serv.poll(&world, Frame(world.tick), Some(IncrB)) {
            for a in acts {
                assert_eq!(world.tick + 1, a.frame.0);
                log::info!(
                    "server_virtual_client got input from server: {:?} w is now {} {} {}",
                    a,
                    world.tick,
                    world.incr_a,
                    world.incr_b,
                );
                world.apply(a.inputs.into_iter().map(|x| x.inp).collect());
            }
        }
        std::thread::sleep(Duration::from_millis(1));
    });
}
