use log::LevelFilter;
use networking::{Client, ConnectConf, Frame, PollResult, Server, ServerConfiguration};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use std::time::Duration;
use Action::*;

#[derive(Serialize, Deserialize, Debug)]
struct World {
    incr_a: u32,
    incr_b: u32,
    tick: u32,
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

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
enum Action {
    DoNothing,
    IncrA,
    IncrB,
}

impl Default for Action {
    fn default() -> Self {
        DoNothing
    }
}

pub fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    serv_c();

    std::thread::sleep(Duration::from_millis(3000));

    let mut client2: Client<World, Action> = Client::connect(ConnectConf {
        name: "client_2".into(),
        addr: Ipv4Addr::LOCALHOST.into(),
        period: Duration::from_millis(20),
        port: None,
        frame_buffer_advance: 10,
    })
    .unwrap();

    let mut world: World;
    loop {
        if let PollResult::GameWorld(_, w) = client2.poll(DoNothing) {
            log::info!(
                "client_2 got world from server: length is {:?}",
                w.pad.len()
            );
            world = w;
            break;
        }
        std::thread::sleep(Duration::from_millis(100))
    }

    for _ in 1usize.. {
        if let PollResult::Input(inp) = client2.poll(IncrA) {
            for inp in inp {
                log::info!(
                    "client_2 got input from server: {:?} w is now {} {} {}",
                    inp,
                    world.tick,
                    world.incr_b,
                    world.incr_b,
                );
                world.apply(inp.into_iter().map(|x| x.inp).collect());
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
        pad: vec![0; 10000000],
    };

    let mut serv = Server::start(ServerConfiguration {
        start_frame: Frame(world.tick),
        period: Duration::from_millis(20),
        port: None,
    })
    .unwrap();

    let mut client1: Client<World, Action> = Client::connect(ConnectConf {
        name: "client_1".into(),
        addr: Ipv4Addr::LOCALHOST.into(),
        port: None,
        period: Duration::from_millis(20),
        frame_buffer_advance: 10,
    })
    .unwrap();

    std::thread::spawn(move || loop {
        serv.poll(&world);
        if let PollResult::Input(acts) = client1.poll(IncrB) {
            for a in acts {
                log::info!(
                    "client_1 got input from server: {:?} w is now {} {} {}",
                    a,
                    world.tick,
                    world.incr_b,
                    world.incr_b,
                );
                world.apply(a.into_iter().map(|x| x.inp).collect());
            }
        }
        std::thread::sleep(Duration::from_millis(1));
    });
}
