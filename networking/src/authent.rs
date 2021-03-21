use crate::packets::{AuthentResponse, ServerReliablePacket};
use crate::{encode, hash_str, Frame, UserID};
use message_io::network::{Endpoint, Network};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, PartialEq, Eq, Copy, Clone, Hash)]
#[repr(transparent)]
pub(crate) struct AuthentID(u32);

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum ClientGameState {
    Downloading,
    CatchingUp,
    Playing,
}

pub(crate) struct Client {
    pub id: UserID,
    pub name: String,
    pub ack: Frame,
    pub reliable: Endpoint,
    pub unreliable: Endpoint,
    pub state: ClientGameState,
}

enum ClientConnectState {
    Connecting {
        reliable: Endpoint,
        unreliable: Option<Endpoint>,
    },
    Connected(Client),
}

pub(crate) struct Authent {
    clients: HashMap<AuthentID, ClientConnectState>,
    addr_to_client: HashMap<SocketAddr, AuthentID>,
    n_connected_clients: u32,
    seq: u32,
}

impl Authent {
    pub fn new() -> Self {
        Self {
            clients: Default::default(),
            addr_to_client: Default::default(),
            n_connected_clients: 0,
            seq: 0,
        }
    }

    pub fn tcp_client_auth(
        &mut self,
        e: Endpoint,
        ack: Frame,
        name: String,
    ) -> Option<AuthentResponse> {
        let state = self.get_client_state_mut(e).unwrap();

        if let ClientConnectState::Connecting {
            reliable,
            unreliable: Some(unreliable),
        } = *state
        {
            log::info!("client authenticated: {}@{}", name, e.addr());
            let hash = hash_str(&name);

            if self.iter().any(|x| x.name == name) {
                return Some(AuthentResponse::Refused {
                    reason: format!("name is already in use: {}", name),
                });
            }

            *self.get_client_state_mut(e).unwrap() = ClientConnectState::Connected(Client {
                id: UserID(hash),
                name,
                ack,
                reliable,
                unreliable,
                state: ClientGameState::Downloading,
            });

            self.n_connected_clients += 1;

            return Some(AuthentResponse::Accepted);
        }
        None
    }

    pub fn udp_connect(&mut self, e: Endpoint, id: AuthentID, net: &mut Network) {
        self.addr_to_client.insert(e.addr(), id);
        if let Some(ClientConnectState::Connecting {
            unreliable,
            reliable,
        }) = self.get_client_state_mut(e)
        {
            *unreliable = Some(e);
            net.send(*reliable, &*encode(&ServerReliablePacket::ReadyForAuth));
        }
    }

    pub fn tcp_connected(&mut self, e: Endpoint, net: &mut Network) {
        log::info!("connected:{}", e);

        let client_id = self.next_client_id();
        self.addr_to_client.insert(e.addr(), client_id);

        self.clients.insert(
            client_id,
            ClientConnectState::Connecting {
                reliable: e,
                unreliable: None,
            },
        );

        net.send(e, &*encode(&ServerReliablePacket::Challenge(client_id)));
    }

    pub fn tcp_disconnected(&mut self, e: Endpoint) -> Option<Client> {
        let id = self.addr_to_client.get(&e.addr())?;
        let client = self.clients.remove(&id)?;
        if let ClientConnectState::Connected(c) = client {
            self.addr_to_client.remove(&c.unreliable.addr());
            self.addr_to_client.remove(&c.reliable.addr());
            self.n_connected_clients -= 1;

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

    fn next_client_id(&mut self) -> AuthentID {
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

impl Default for Authent {
    fn default() -> Self {
        Self::new()
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
