use std::fmt::Debug;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

use message_io::events::EventQueue;
use message_io::network::{Endpoint, NetEvent, Network, Transport};
use serde::__private::PhantomData;
use serde::de::DeserializeOwned;
use serde::Serialize;

use client_playout::ClientPlayoutBuffer;

use crate::packets::{
    AuthentResponse, ClientReliablePacket, ClientUnreliablePacket, ServerReliablePacket,
    ServerUnreliablePacket,
};
use crate::worldsend::WorldReceive;
use crate::{decode, encode, Frame, MergedInputs, PlayerInput, DEFAULT_PORT};

mod client_playout;

pub enum PollResult<W, I> {
    Wait(I),
    Input(Vec<Vec<I>>),
    GameWorld(W),
    Error,
}

#[allow(clippy::large_enum_variant)]
enum ClientState<I> {
    Connecting,
    Downloading(WorldReceive),
    CatchingUp { next_inputs: Option<Vec<Vec<I>>> },
    Playing(ClientPlayoutBuffer),
}

pub struct Client<WORLD: DeserializeOwned, INPUT: Serialize + DeserializeOwned + Default> {
    network: Network,
    events: EventQueue<NetEvent>,
    tcp: Endpoint,
    udp: Endpoint,

    name: String,

    clock: Instant,
    period: Duration,
    state: ClientState<INPUT>,
    frame_buffer_advance: u32,

    _phantom: PhantomData<(INPUT, WORLD)>,
}

pub struct ConnectConf {
    pub name: String,
    pub addr: IpAddr,
    pub port: Option<u16>,
    pub period: Duration,
    pub frame_buffer_advance: u32,
}

impl<W: DeserializeOwned, I: Serialize + DeserializeOwned + Default + Debug> Client<W, I> {
    pub fn connect(conf: ConnectConf) -> io::Result<Self> {
        let (mut network, events) = Network::split();
        let addr = conf.addr;
        let port = conf.port.unwrap_or(DEFAULT_PORT);
        let (tcp, _) = network.connect(Transport::FramedTcp, SocketAddr::new(addr, port))?;
        let (udp, _) = network.connect(Transport::Udp, SocketAddr::new(addr, port + 1))?;

        Ok(Self {
            network,
            events,
            tcp,
            udp,
            state: ClientState::Connecting,
            name: conf.name,
            frame_buffer_advance: conf.frame_buffer_advance,
            period: conf.period,
            clock: Instant::now(),
            _phantom: Default::default(),
        })
    }

    pub fn poll(&mut self, input: I) -> PollResult<W, I> {
        while let Some(x) = self.events.try_receive() {
            match x {
                NetEvent::Message(e, m) => {
                    if e.resource_id().adapter_id() == Transport::FramedTcp.id() {
                        let packet = decode(&*m).expect("invalid reliable packet");
                        self.message_reliable(packet);
                    } else {
                        let packet = decode(&*m).expect("invalid reliable packet");
                        self.message_unreliable(packet)
                    }
                }
                NetEvent::Connected(e) => {
                    log::info!("connected {}", e)
                }
                NetEvent::Disconnected(e) => {
                    log::info!("disconnected {}", e)
                }
            }
        }

        match self.state {
            ClientState::Connecting => {
                return PollResult::Wait(input);
            }
            ClientState::Downloading(ref mut recv) => {
                if let Some(world) = recv.check_finished() {
                    self.state = ClientState::CatchingUp { next_inputs: None };
                    self.network
                        .send(self.tcp, &*encode(&ClientReliablePacket::BeginCatchUp));
                    return match decode(&*world) {
                        Ok(x) => PollResult::GameWorld(x),
                        Err(e) => {
                            log::error!("couldn't decode world: {}", e);
                            return PollResult::Error;
                        }
                    };
                }
            }
            ClientState::CatchingUp {
                ref mut next_inputs,
            } => {
                if let Some(x) = next_inputs.take() {
                    log::info!("{} catching up consumed inputs, asking for more", self.name);
                    self.network
                        .send(self.tcp, &*encode(&ClientReliablePacket::CatchUpAck));
                    return PollResult::Input(x);
                }
                return PollResult::Wait(input);
            }
            ClientState::Playing(ref mut buffer) => {
                if self.clock.elapsed() < self.period {
                    return PollResult::Wait(input);
                }

                let mut inp = Some(&input);
                let mut mk_input = || {
                    let d = Default::default();
                    let v = inp.take().unwrap_or(&d);
                    let serialized = encode(&v);
                    PlayerInput(serialized)
                };

                let advance = buffer.advance();

                let fba = self.frame_buffer_advance;
                let to_consume = match advance {
                    0 => 0,
                    _ if (1..=fba).contains(&advance) => 1,
                    _ if (fba + 1..=fba * 2).contains(&advance) => 2,
                    _ if (fba * 2 + 1..=fba * 3).contains(&advance) => 3,
                    _ => 4,
                };

                if to_consume > 0 {
                    self.clock = Instant::now();
                    let net = &mut self.network;
                    let udp = self.udp;
                    let name = &self.name;
                    let ack_frame = buffer.consumed_frame() + Frame(advance);

                    let multi = (0..to_consume)
                        .map(move |_| {
                            log::info!("{}: sending inputs to server", name);
                            let (inp, pack) = buffer.try_consume(&mut mk_input).unwrap();
                            net.send(
                                udp,
                                &*encode(&ClientUnreliablePacket::Input {
                                    input: pack,
                                    ack_frame,
                                }),
                            );
                            decode_merged(inp)
                        })
                        .collect();
                    return PollResult::Input(multi);
                }
            }
        }

        PollResult::Wait(input)
    }

    fn message_reliable(&mut self, p: ServerReliablePacket) -> Option<()> {
        match p {
            ServerReliablePacket::ReadyForAuth => {
                log::info!("{}: received ready for auth", self.name);
                let connect = ClientReliablePacket::Connect {
                    name: self.name.clone(),
                };
                self.network.send(self.tcp, &*encode(&connect));
            }
            ServerReliablePacket::WorldSend(fragment) => {
                log::info!("{}: received world fragment", self.name);

                if let ClientState::Downloading(ref mut wr) = self.state {
                    wr.handle(fragment, &mut self.network, self.tcp);
                } else {
                    log::error!("received world but was not downloading.. weird");
                }
            }
            ServerReliablePacket::Challenge(challenge) => {
                log::info!("{}: received challenge", self.name);

                self.network.send(
                    self.udp,
                    &*encode(&ClientUnreliablePacket::Connection(challenge)),
                );
            }
            ServerReliablePacket::AuthentResponse(r) => match r {
                AuthentResponse::Accepted => {
                    log::info!(
                        "{}: authent response is accepted. asking for world",
                        self.name
                    );
                    self.state = ClientState::Downloading(WorldReceive::default());
                    self.network
                        .send(self.tcp, &*encode(&ClientReliablePacket::WorldAck));
                }
                AuthentResponse::Refused { reason } => {
                    log::error!("authent refused :( reason: {}", reason)
                }
            },
            ServerReliablePacket::CatchUp { inputs } => {
                log::info!("{}: received catch up inputs", self.name);

                if let ClientState::CatchingUp {
                    ref mut next_inputs,
                } = self.state
                {
                    *next_inputs = Some(inputs.into_iter().map(decode_merged).collect());
                } else {
                    log::error!("received catching up inputs but was not catching up.. weird");
                }
            }
            ServerReliablePacket::ReadyToPlay { start_frame } => {
                log::info!("{}: received ready to play on {:?}", self.name, start_frame);
                if let ClientState::CatchingUp { next_inputs: None } = self.state {
                    self.state = ClientState::Playing(ClientPlayoutBuffer::new(start_frame, 3));
                    self.network
                        .send(self.tcp, &*encode(&ClientReliablePacket::ReadyToPlayAck));
                } else {
                    log::error!(
                        "received ready to play but was still catching up or connecting.. weird"
                    );
                }
            }
        }
        None
    }

    fn message_unreliable(&mut self, p: ServerUnreliablePacket) {
        match p {
            ServerUnreliablePacket::Input(inp) => {
                log::info!(
                    "{}: received inputs from server. {}->{}",
                    self.name,
                    inp.first().unwrap().0 .0,
                    inp.last().unwrap().0 .0
                );
                for (frame, inp) in inp {
                    if let ClientState::Playing(ref mut buffer) = self.state {
                        let _ = buffer.insert_serv_input(frame, inp);
                    }
                }
            }
        }
    }
}

fn decode_merged<I: DeserializeOwned>(x: MergedInputs) -> Vec<I> {
    x.into_iter().flat_map(|x| decode(&x.0).ok()).collect()
}
