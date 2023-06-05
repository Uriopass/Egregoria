use crate::connections::Connections;
use crate::packets::{AuthentResponse, ServerReliablePacket, ServerUnreliablePacket};
use crate::{encode, hash_str, Frame, UserID};
use common::{FastMap, FastSet};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use std::time::Duration;

#[derive(Serialize, Deserialize, PartialEq, Eq, Copy, Clone, Hash, Debug)]
#[repr(transparent)]
pub(crate) struct AuthentID(pub(crate) u32);

impl Display for AuthentID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl AuthentID {
    pub const VIRTUAL_ID: AuthentID = AuthentID(0);
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum ClientGameState {
    Downloading,
    CatchingUp,
    Playing,
}

pub(crate) struct Client {
    pub id: AuthentID,
    #[allow(dead_code)]
    pub uid: UserID,
    pub name: String,
    pub ack: Frame,
    pub udp_addr: SocketAddr,
    pub tcp_addr: SocketAddr,
    pub state: ClientGameState,
}

enum ClientConnectState {
    Connecting {
        id: AuthentID,
        tcp_addr: SocketAddr,
        udp_addr: Option<SocketAddr>,
    },
    Connected(Client),
}

pub(crate) struct Authent {
    names: FastSet<String>,
    clients: FastMap<AuthentID, ClientConnectState>,
    addr_to_client: FastMap<SocketAddr, AuthentID>,
    n_connected_clients: u32,
    seq: u32,
    version: String,
}

impl Authent {
    pub fn new(version: String) -> Self {
        Self {
            names: Default::default(),
            clients: Default::default(),
            addr_to_client: Default::default(),
            n_connected_clients: 0,
            seq: 1,
            version,
        }
    }

    /// returns true if the player was already registered
    pub fn register(&mut self, name: String) -> bool {
        !self.names.insert(name)
    }

    pub fn tcp_client_auth(
        &mut self,
        addr: SocketAddr,
        ack: Frame,
        name: String,
        version: String,
        period: Duration,
    ) -> Option<AuthentResponse> {
        let v = self.get_client_state_mut(addr)?;

        if let ClientConnectState::Connecting {
            id,
            tcp_addr,
            udp_addr: Some(udp_addr),
        } = *v
        {
            log::info!("client authenticated: {}@{}", name, addr);
            let hash = hash_str(&name);

            if self.register(name.clone()) {
                return Some(AuthentResponse::Refused {
                    reason: format!("name is already in use: {name}"),
                });
            }

            if version != self.version {
                return Some(AuthentResponse::Refused {
                    reason: format!(
                        "Incompatible versions: serv: {} vs client: {}",
                        self.version, version
                    ),
                });
            }

            // Unwrap ok: already checked right before
            *self.get_client_state_mut(tcp_addr).unwrap() = ClientConnectState::Connected(Client {
                id,
                uid: UserID(hash),
                name,
                ack,

                udp_addr,
                tcp_addr,
                state: ClientGameState::Downloading,
            });

            self.n_connected_clients += 1;

            return Some(AuthentResponse::Accepted { id, period });
        }
        None
    }

    pub fn udp_connect(&mut self, addr: SocketAddr, id: AuthentID, net: &Connections) {
        log::info!("udp connect: {}", addr);
        self.addr_to_client.insert(addr, id);
        if let Some(ClientConnectState::Connecting { udp_addr, .. }) =
            self.get_client_state_mut(addr)
        {
            *udp_addr = Some(addr);
            net.send_udp(addr, encode(&ServerUnreliablePacket::ReadyForAuth));
            net.send_udp(addr, encode(&ServerUnreliablePacket::ReadyForAuth));
            net.send_udp(addr, encode(&ServerUnreliablePacket::ReadyForAuth));
        }
    }

    pub fn tcp_connected(&mut self, tcp_addr: SocketAddr, net: &Connections) {
        log::info!("connected: {}", tcp_addr);

        let id = self.next_auth_id();
        self.addr_to_client.insert(tcp_addr, id);

        self.clients.insert(
            id,
            ClientConnectState::Connecting {
                id,
                tcp_addr,
                udp_addr: None,
            },
        );

        net.send_tcp(tcp_addr, encode(&ServerReliablePacket::Challenge(id)));
    }

    pub fn disconnected(&mut self, tcp_addr: SocketAddr) -> Option<Client> {
        let id = self.addr_to_client.remove(&tcp_addr)?;
        let client = self.clients.remove(&id)?;

        if let ClientConnectState::Connecting { .. } = client {
            return None;
        }
        if let ClientConnectState::Connected(c) = client {
            self.addr_to_client.remove(&c.udp_addr);
            self.n_connected_clients -= 1;
            self.names.remove(&c.name);

            return Some(c);
        }
        None
    }

    pub fn get_client(&self, addr: SocketAddr) -> Option<&Client> {
        self.addr_to_client
            .get(&addr)
            .and_then(|x| self.clients.get(x))
            .and_then(ClientConnectState::as_connected)
    }

    pub fn get_client_mut(&mut self, addr: SocketAddr) -> Option<&mut Client> {
        self.get_client_state_mut(addr)
            .and_then(ClientConnectState::as_connected_mut)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Client> + Clone {
        self.clients
            .values()
            .filter_map(ClientConnectState::as_connected)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Client> {
        self.clients
            .values_mut()
            .filter_map(ClientConnectState::as_connected_mut)
    }

    pub fn iter_playing(&self) -> impl Iterator<Item = &Client> + Clone {
        self.iter().filter(|x| x.state == ClientGameState::Playing)
    }

    fn next_auth_id(&mut self) -> AuthentID {
        self.seq += 1;
        AuthentID(self.seq)
    }

    fn get_client_state_mut(&mut self, addr: SocketAddr) -> Option<&mut ClientConnectState> {
        let clients = &mut self.clients;
        self.addr_to_client
            .get(&addr)
            .and_then(move |x| clients.get_mut(x))
    }
}

impl ClientConnectState {
    pub fn as_connected(&self) -> Option<&Client> {
        if let ClientConnectState::Connected(c) = self {
            Some(c)
        } else {
            None
        }
    }

    pub fn as_connected_mut(&mut self) -> Option<&mut Client> {
        if let ClientConnectState::Connected(c) = self {
            Some(c)
        } else {
            None
        }
    }
}
