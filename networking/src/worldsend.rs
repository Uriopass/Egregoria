use crate::authent::{Client, ClientGameState};
use crate::packets::{ClientReliablePacket, ServerReliablePacket, WorldDataFragment};
use crate::{encode, UserID, MAX_WORLDSEND_PACKET_SIZE};
use common::FastMap;
use message_io::network::{Endpoint, Network};

#[derive(Eq, PartialEq)]
enum WorldSendStatus {
    WaitingForAck,
    ReadyToSend,
    WaitingForFinalAck,
    Over,
}

struct WorldSendState {
    data: Vec<u8>,
    sent: usize,
    status: WorldSendStatus,
}

#[derive(Default)]
pub(crate) struct WorldSend {
    send_state: FastMap<UserID, WorldSendState>,
}

impl WorldSend {
    pub fn begin_send(&mut self, c: &Client, data: Vec<u8>) {
        self.send_state.insert(
            c.id,
            WorldSendState {
                data,
                sent: 0,
                status: WorldSendStatus::WaitingForAck,
            },
        );
    }

    pub fn ack(&mut self, c: &Client) {
        if let Some(state) = self.send_state.get_mut(&c.id) {
            if state.status == WorldSendStatus::WaitingForAck {
                state.status = WorldSendStatus::ReadyToSend;
            }
            if state.status == WorldSendStatus::WaitingForFinalAck {
                state.status = WorldSendStatus::Over
            }
        } else {
            log::error!("ack ing a non existing world send");
        }
    }

    pub fn update(&mut self, c: &mut Client, net: &mut Network) {
        if let Some(state) = self.send_state.get_mut(&c.id) {
            if state.status == WorldSendStatus::Over {
                self.send_state.remove(&c.id);
                c.state = ClientGameState::CatchingUp;
                return;
            }
            if state.status != WorldSendStatus::ReadyToSend {
                return;
            }

            state.status = WorldSendStatus::WaitingForAck;

            let to_send = MAX_WORLDSEND_PACKET_SIZE.min(state.data.len() - state.sent);
            let is_over = to_send < MAX_WORLDSEND_PACKET_SIZE;

            net.send(
                c.reliable,
                &*encode(&ServerReliablePacket::WorldSend(WorldDataFragment {
                    is_over,
                    data: Vec::from(&state.data[state.sent..state.sent + to_send]),
                })),
            );

            log::info!("{}: sending world fragment", c.name);

            state.sent += to_send;
        } else {
            log::error!("updating a non existing world send");
        }
    }

    pub fn disconnected(&mut self, id: UserID) {
        self.send_state.remove(&id);
    }
}

#[derive(Default)]
pub(crate) struct WorldReceive {
    data_so_far: Vec<u8>,
    is_over: bool,
}

impl WorldReceive {
    pub fn handle(&mut self, fragment: WorldDataFragment, net: &mut Network, tcp: Endpoint) {
        self.data_so_far.extend(fragment.data);
        self.is_over = fragment.is_over;
        net.send(tcp, &*encode(&ClientReliablePacket::WorldAck));
    }

    pub fn check_finished(&mut self) -> Option<Vec<u8>> {
        if self.is_over {
            Some(std::mem::take(&mut self.data_so_far))
        } else {
            None
        }
    }
}
