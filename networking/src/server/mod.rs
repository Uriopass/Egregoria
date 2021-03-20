use std::io;
use std::marker::PhantomData;
use std::time::{Duration, Instant};

use message_io::events::EventQueue;
use message_io::network::{Endpoint, NetEvent, Network, Transport};
use serde::Serialize;

use crate::authent::{Authent, ClientGameState};
use crate::catchup::CatchUp;
use crate::packets::{
    AuthentResponse, ClientReliablePacket, ClientUnreliablePacket, ServerReliablePacket,
    ServerUnreliablePacket,
};
use crate::server::server_playout::{InsertResult, ServerPlayoutBuffer};
use crate::worldsend::WorldSend;
use crate::{decode, encode, Frame, DEFAULT_PORT};

mod server_playout;

pub struct Server<WORLD: Serialize> {
    network: Network,
    events: EventQueue<NetEvent>,

    authent: Authent,
    buffer: ServerPlayoutBuffer,
    lag: Frame,
    catchup: CatchUp,
    worldsend: WorldSend,

    clock: Instant,
    period: Duration,

    serv_tick: Frame,
    _phantom: PhantomData<WORLD>,
}

pub struct ServerConfiguration {
    pub start_frame: Frame,
    pub lag: Frame,
    pub period: Duration,
    pub port: Option<u16>,
}

impl<WORLD: Serialize> Server<WORLD> {
    pub fn start(conf: ServerConfiguration) -> io::Result<Self> {
        let (mut network, events) = Network::split();

        let port = conf.port.unwrap_or(DEFAULT_PORT);
        network.listen(Transport::FramedTcp, format!("127.0.0.1:{}", port))?;
        network.listen(Transport::Udp, format!("127.0.0.1:{}", port + 1))?;

        Ok(Self {
            network,
            events,
            lag: conf.lag,
            period: conf.period,
            serv_tick: conf.start_frame,
            buffer: ServerPlayoutBuffer::new(conf.start_frame),
            clock: Instant::now(),
            authent: Authent::default(),
            catchup: CatchUp::default(),
            worldsend: Default::default(),
            _phantom: PhantomData::default(),
        })
    }

    pub fn poll(&mut self, world: &WORLD) {
        self.send_merged_inputs();
        self.send_long_running();
        while let Some(ev) = self.events.try_receive() {
            match ev {
                NetEvent::Message(e, data) => {
                    if is_reliable(&e) {
                        let packet = match decode::<ClientReliablePacket>(&data) {
                            Ok(x) => x,
                            Err(_) => break,
                        };

                        let _ = self.message_reliable(e, packet, world);
                    } else {
                        let packet = match decode::<ClientUnreliablePacket>(&data) {
                            Ok(x) => x,
                            Err(_) => break,
                        };

                        let _ = self.message_unreliable(e, packet);
                    }
                }
                NetEvent::Connected(e) => self.tcp_connected(e),
                NetEvent::Disconnected(e) => self.tcp_disconnected(e),
            }
        }
    }

    fn send_merged_inputs(&mut self) {
        let n_playing = self.authent.iter_playing().count();

        if n_playing == 0 {
            return;
        }

        if self.clock.elapsed() > self.period {
            self.serv_tick.incr();
            self.clock = Instant::now();
        }

        let clients_playing = self.authent.iter_playing();

        if let Some((consumed_inputs, inputs)) = self.buffer.try_consume(
            clients_playing.clone().map(|c| c.ack),
            self.serv_tick > self.buffer.consumed_frame,
            n_playing as usize,
        ) {
            for (playing, packet) in clients_playing.zip(inputs) {
                self.network.send(
                    playing.unreliable,
                    &*encode(&ServerUnreliablePacket::Input(packet)),
                );
            }
            self.catchup
                .add_merged_inputs(self.buffer.consumed_frame, consumed_inputs)
        }
    }

    fn send_long_running(&mut self) {
        for c in self.authent.iter_mut() {
            match c.state {
                ClientGameState::Downloading => {
                    self.worldsend.update(c, &mut self.network);
                }
                ClientGameState::CatchingUp => {
                    self.catchup.update(c, &mut self.network);
                }
                _ => {}
            }
        }
    }

    fn message_unreliable(&mut self, e: Endpoint, packet: ClientUnreliablePacket) -> Option<()> {
        match packet {
            ClientUnreliablePacket::Input { input, ack_frame } => {
                let client = self.authent.get_client_mut(e)?;

                log::info!("{}: received inputs {:?}", client.name, ack_frame);
                client.ack = ack_frame;

                for (frame, input) in input {
                    let res = self.buffer.insert_input(frame + self.lag, client.id, input);
                    if matches!(res, InsertResult::TooFarAhead) {
                        log::error!("too far ahead");
                    }
                }
            }
            ClientUnreliablePacket::Connection(id) => {
                self.authent.udp_connect(e, id, &mut self.network);
            }
        }
        Some(())
    }

    fn message_reliable(
        &mut self,
        e: Endpoint,
        packet: ClientReliablePacket,
        world: &WORLD,
    ) -> Option<()> {
        match packet {
            ClientReliablePacket::Connect { name } => {
                let auth_r = self
                    .authent
                    .tcp_client_auth(e, self.buffer.consumed_frame, name)?;
                let accepted = matches!(auth_r, AuthentResponse::Accepted);
                self.network
                    .send(e, &*encode(&ServerReliablePacket::AuthentResponse(auth_r)));

                if accepted {
                    let c = self.authent.get_client(e)?;
                    self.worldsend.begin_send(c, encode(world));
                    self.catchup
                        .begin_remembering(self.buffer.consumed_frame, c);

                    self.authent.get_client_mut(e)?.state = ClientGameState::Downloading;
                } else {
                    self.network.remove(e.resource_id());
                }
            }
            ClientReliablePacket::BeginCatchUp => {
                let c = self.authent.get_client_mut(e)?;
                log::info!("client {} ready to catch up", c.name);
                c.state = ClientGameState::CatchingUp;
                self.catchup.ack(c);
            }
            ClientReliablePacket::CatchUpAck => {
                let c = self.authent.get_client(e)?;
                log::info!("client {} ack", c.name);
                self.catchup.ack(c);
            }
            ClientReliablePacket::WorldAck => {
                let c = self.authent.get_client(e)?;
                log::info!("client {} ack", c.name);
                self.worldsend.ack(c);
            }
            ClientReliablePacket::ReadyToPlayAck => {
                self.authent.get_client_mut(e)?.state = ClientGameState::Playing;
            }
        }
        Some(())
    }

    fn tcp_connected(&mut self, e: Endpoint) {
        self.authent.tcp_connected(e, &mut self.network)
    }

    fn tcp_disconnected(&mut self, e: Endpoint) {
        if let Some(c) = self.authent.tcp_disconnected(e) {
            self.buffer.disconnected(c.id);
            self.catchup.disconnected(c.id);
            self.worldsend.disconnected(c.id);
        }
    }
}

fn is_reliable(e: &Endpoint) -> bool {
    e.resource_id().adapter_id() == Transport::FramedTcp.id()
}
