use std::io;
use std::time::Duration;

use message_io::network::{Endpoint, NetEvent, NetworkController, NetworkProcessor, Transport};
use serde::Serialize;

use crate::authent::{Authent, AuthentID, ClientGameState};
use crate::catchup::CatchUp;
use crate::client::FrameInputs;
use crate::packets::{
    AuthentResponse, ClientReliablePacket, ClientUnreliablePacket, ServerReliablePacket,
    ServerUnreliablePacket,
};
use crate::server::server_playout::ServerPlayoutBuffer;
use crate::worldsend::WorldSend;
use crate::{decode, decode_merged, encode, Frame, PhantomSendSync, PlayerInput, DEFAULT_PORT};
use common::timestep::Timestep;
use serde::de::DeserializeOwned;
use std::net::SocketAddr;

mod server_playout;

pub struct ServerConfiguration {
    pub start_frame: Frame,
    pub period: Duration,
    pub port: Option<u16>,
    pub virtual_client: Option<VirtualClientConf>,
    /// Checks if client has same version or refuses authent otherwise
    pub version: String,
    /// Always run, even when everyone is disconnected
    pub always_run: bool,
}

pub struct VirtualClientConf {
    pub name: String,
}

pub enum ServerPollResult<I> {
    Wait(Option<I>),
    Input(Vec<FrameInputs<I>>),
}

struct VirtualClient {
    name: String,
}

pub struct Server<WORLD: Serialize, INPUT> {
    network: NetworkController,
    events: Option<NetworkProcessor>,

    authent: Authent,
    v_client: Option<VirtualClient>,
    next_inputs: Vec<FrameInputs<INPUT>>,
    buffer: ServerPlayoutBuffer,
    catchup: CatchUp,
    worldsend: WorldSend,

    step: Timestep,
    always_run: bool,

    _phantom: PhantomSendSync<(WORLD, INPUT)>,

    tcp_addr: SocketAddr,
    udp_addr: SocketAddr,
}

impl<WORLD: 'static + Serialize, INPUT: Serialize + DeserializeOwned> Server<WORLD, INPUT> {
    pub fn start(conf: ServerConfiguration) -> io::Result<Self> {
        let (network, events) = message_io::network::split();

        let port = conf.port.unwrap_or(DEFAULT_PORT);
        let (_, tcp_addr) = network.listen(Transport::FramedTcp, format!("0.0.0.0:{}", port))?;
        let (_, udp_addr) = network.listen(Transport::Udp, format!("0.0.0.0:{}", port))?;

        let mut authent = Authent::new(conf.version);
        let v_client = conf.virtual_client.map(|c| VirtualClient { name: c.name });
        if let Some(ref v_client) = v_client {
            authent.register(v_client.name.clone());
        }

        Ok(Self {
            network,
            events: Some(events),
            step: Timestep::new(conf.period),
            buffer: ServerPlayoutBuffer::new(conf.start_frame),
            v_client,
            authent,
            catchup: CatchUp::default(),
            worldsend: Default::default(),
            _phantom: Default::default(),
            tcp_addr,
            udp_addr,
            always_run: conf.always_run,
            next_inputs: vec![],
        })
    }

    pub fn poll(
        &mut self,
        world: &WORLD,
        frame: Frame,
        local_inputs: Option<INPUT>,
    ) -> ServerPollResult<INPUT> {
        let mut called = true;
        let mut events = self.events.take().unwrap();
        while called {
            called = false;
            events.process_poll_event(Some(Duration::ZERO), |ev| {
                called = true;
                match ev {
                    NetEvent::Message(e, data) => match is_reliable(&e) {
                        true => {
                            if let Some(packet) = decode::<ClientReliablePacket>(&data) {
                                let _ = self.message_reliable(e, packet, world, frame);
                            } else {
                                log::error!("client sent invalid reliable packet");
                            }
                        }
                        false => {
                            if let Some(packet) = decode::<ClientUnreliablePacket>(&data) {
                                let _ = self.message_unreliable(e, packet);
                            } else {
                                log::error!("client sent invalid unreliable packet");
                            }
                        }
                    },
                    NetEvent::Accepted(e, _) => self.tcp_connected(e),
                    NetEvent::Disconnected(e) => self.tcp_disconnected(e),
                    NetEvent::Connected(_, _) => {}
                }
            });
        }
        self.events = Some(events);

        self.send_merged_inputs();
        self.send_long_running();

        if !self.next_inputs.is_empty() {
            if self.v_client.is_some() {
                if let Some(inp) = local_inputs {
                    self.buffer.insert_input(
                        AuthentID::VIRTUAL_ID,
                        self.buffer.consumed_frame.incred(),
                        PlayerInput(encode(&inp)),
                    );
                }
            }
            return ServerPollResult::Input(std::mem::take(&mut self.next_inputs));
        }
        ServerPollResult::Wait(local_inputs)
    }

    fn send_merged_inputs(&mut self) {
        let n_playing = self.authent.iter_playing().count() + self.v_client.is_some() as usize;

        if n_playing == 0 && !self.always_run {
            return;
        }

        self.step.prepare_frame(1);

        while self.step.tick() {
            let buffer = &self.buffer;
            let to_disconnect = self
                .authent
                .iter_playing()
                .filter(|c| buffer.lag(c.ack).is_none())
                .map(|c| (c.reliable, c.ack, c.name.clone()))
                .collect::<Vec<_>>();
            for (reliable, ack, name) in to_disconnect {
                log::warn!(
                    "disconnecting {} because it is too late. consumed is {:?} while he is at {:?}",
                    name,
                    self.buffer.consumed_frame,
                    ack,
                );
                self.disconnect(reliable);
            }

            let clients_playing = self.authent.iter_playing();

            let (consumed_inputs, inputs) =
                self.buffer.consume(clients_playing.clone().map(|c| c.ack));

            for (playing, packet) in clients_playing.zip(inputs) {
                self.network.send(
                    playing.unreliable,
                    &encode(&ServerUnreliablePacket::Input(packet)),
                );
            }

            self.next_inputs.push(decode_merged(
                AuthentID::VIRTUAL_ID,
                consumed_inputs.clone(),
                self.buffer.consumed_frame,
            ));

            self.catchup
                .add_merged_inputs(self.buffer.consumed_frame, consumed_inputs);
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
            ClientUnreliablePacket::Input { input } => {
                let client = self.authent.get_client_mut(e)?;

                //log::info!("{}: received inputs {:?}", client.name, ack_frame);

                for (frame, input) in input {
                    client.ack = client.ack.max(frame);
                    self.buffer.insert_input(client.id, frame, input);
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
        w: &WORLD,
        w_frame: Frame,
    ) -> Option<()> {
        match packet {
            ClientReliablePacket::Connect { name, version } => {
                let auth_r = self.authent.tcp_client_auth(
                    e,
                    self.buffer.consumed_frame,
                    name,
                    version,
                    self.step.period,
                )?;

                self.network.send(
                    e,
                    &encode(&ServerReliablePacket::AuthentResponse(auth_r.clone())),
                );

                match auth_r {
                    AuthentResponse::Accepted { .. } => {
                        let c = self.authent.get_client(e)?;
                        assert_eq!(self.buffer.consumed_frame, w_frame);
                        self.worldsend.begin_send(c, encode(&w), w_frame);
                        self.catchup
                            .begin_remembering(self.buffer.consumed_frame, c);

                        self.authent.get_client_mut(e)?.state = ClientGameState::Downloading;
                    }
                    AuthentResponse::Refused { reason } => {
                        log::error!("refused authent because: {}", reason);
                        self.network.remove(e.resource_id());
                    }
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
                log::info!("client {} world rcv acked", c.name);
                self.worldsend.ack(c);
            }
        }
        Some(())
    }

    fn tcp_connected(&mut self, e: Endpoint) {
        self.authent.tcp_connected(e, &mut self.network)
    }

    fn tcp_disconnected(&mut self, e: Endpoint) {
        self.disconnect(e);
    }

    pub fn describe(&self) -> String {
        let mut s = "".to_string();
        s += &*format!("listening to {} (tcp)\n", self.tcp_addr);
        s += &*format!("         and {} (udp)\n", self.udp_addr);

        s += "Users:\n";
        if let Some(ref c) = self.v_client {
            s += &*format!("{}: Playing...\n", c.name)
        }
        for c in self.authent.iter() {
            s += &*format!("{}: {:?}...\n", c.name, c.state);
        }
        s
    }

    fn disconnect(&mut self, e: Endpoint) {
        if e.resource_id().adapter_id() == Transport::Udp.id() {
            log::error!("trying to disconnect udp endpoint");
            return;
        }
        if let Some(c) = self.authent.disconnected(e) {
            log::info!("player {} disconnected", c.name);
            self.buffer.disconnected(c.id);
            self.catchup.disconnected(c.id);
            self.worldsend.disconnected(c.id);
        }
    }
}

fn is_reliable(e: &Endpoint) -> bool {
    e.resource_id().adapter_id() == Transport::FramedTcp.id()
}
