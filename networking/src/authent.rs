use crate::packets::{AuthentResponse, ServerReliablePacket, ServerUnreliablePacket};
use crate::{encode, hash_str, Frame, UserID};
use common::{FastMap, FastSet};
use message_io::network::{Endpoint, Network};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;

#[derive(Serialize, Deserialize, PartialEq, Eq, Copy, Clone, Hash, Debug)]
#[repr(transparent)]
pub(crate) struct AuthentID(u32);

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
    pub uid: UserID,
    pub name: String,
    pub ack: Frame,
    pub reliable: Endpoint,
    pub unreliable: Endpoint,
    pub state: ClientGameState,
}

enum ClientConnectState {
    Connecting {
        id: AuthentID,
        reliable: Endpoint,
        unreliable: Option<Endpoint>,
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
        e: Endpoint,
        ack: Frame,
        name: String,
        version: String,
        period: Duration,
    ) -> Option<AuthentResponse> {
        let v = self.get_client_state_mut(e)?;

        if let ClientConnectState::Connecting {
            id,
            reliable,
            unreliable: Some(unreliable),
        } = *v
        {
            log::info!("client authenticated: {}@{}", name, e.addr());
            let hash = hash_str(&name);

            if self.register(name.clone()) {
                return Some(AuthentResponse::Refused {
                    reason: format!("name is already in use: {}", name),
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
            *self.get_client_state_mut(e).unwrap() = ClientConnectState::Connected(Client {
                id,
                uid: UserID(hash),
                name,
                ack,
                reliable,
                unreliable,
                state: ClientGameState::Downloading,
            });

            self.n_connected_clients += 1;

            return Some(AuthentResponse::Accepted { id, period });
        }
        None
    }

    pub fn udp_connect(&mut self, e: Endpoint, id: AuthentID, net: &mut Network) {
        self.addr_to_client.insert(e.addr(), id);
        if let Some(ClientConnectState::Connecting { unreliable, .. }) =
            self.get_client_state_mut(e)
        {
            *unreliable = Some(e);
            net.send(e, &*encode(&ServerUnreliablePacket::ReadyForAuth));
            net.send(e, &*encode(&ServerUnreliablePacket::ReadyForAuth));
            net.send(e, &*encode(&ServerUnreliablePacket::ReadyForAuth));
        }
    }

    pub fn tcp_connected(&mut self, e: Endpoint, net: &mut Network) {
        log::info!("connected:{}", e);

        let id = self.next_auth_id();
        self.addr_to_client.insert(e.addr(), id);

        self.clients.insert(
            id,
            ClientConnectState::Connecting {
                id,
                reliable: e,
                unreliable: None,
            },
        );

        net.send(e, &*encode(&ServerReliablePacket::Challenge(id)));
    }

    pub fn disconnected(&mut self, e: Endpoint) -> Option<Client> {
        let id = self.addr_to_client.remove(&e.addr())?;
        let client = self.clients.remove(&id)?;

        if let ClientConnectState::Connecting {
            unreliable: Some(unreliable),
            ..
        } = client
        {
            self.addr_to_client.remove(&unreliable.addr());
            return None;
        }
        if let ClientConnectState::Connected(c) = client {
            self.addr_to_client.remove(&c.unreliable.addr());
            self.n_connected_clients -= 1;
            self.names.remove(&c.name);

            return Some(c);
        }
        None
    }

    pub fn get_client(&self, e: Endpoint) -> Option<&Client> {
        self.addr_to_client
            .get(&e.addr())
            .and_then(|x| self.clients.get(x))
            .and_then(ClientConnectState::as_connected)
    }

    pub fn get_client_mut(&mut self, e: Endpoint) -> Option<&mut Client> {
        self.get_client_state_mut(e)
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

    fn get_client_state_mut(&mut self, e: Endpoint) -> Option<&mut ClientConnectState> {
        let clients = &mut self.clients;
        self.addr_to_client
            .get(&e.addr())
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
