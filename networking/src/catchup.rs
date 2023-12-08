use crate::authent::{Client, ClientGameState};
use crate::connections::Connections;
use crate::packets::ServerReliablePacket;
use crate::{encode, AuthentID, Frame, MergedInputs};
use common::FastMap;

struct CatchUpState {
    inputs: Vec<MergedInputs>,
    sent: usize,
    from: Frame,
    ready: bool,
}

#[derive(Default)]
pub(crate) struct CatchUp {
    frame_history: FastMap<AuthentID, CatchUpState>,
}

impl CatchUp {
    pub fn begin_remembering(&mut self, from: Frame, c: &Client) {
        let v = self.frame_history.insert(
            c.id,
            CatchUpState {
                inputs: vec![],
                sent: 0,
                from,
                ready: false,
            },
        );

        if v.is_some() {
            log::error!("client was already catching up ??")
        }
    }

    pub fn add_merged_inputs(&mut self, frame: Frame, inp: MergedInputs) {
        for v in self.frame_history.values_mut() {
            if frame.0 != v.from.0 + 1 + v.inputs.len() as u64 {
                log::error!("wrong input for catch up !!!")
            }
            v.inputs.push(inp.clone())
        }
    }

    pub fn ack(&mut self, c: &Client) {
        if let Some(x) = self.frame_history.get_mut(&c.id) {
            x.ready = true;
        }
    }

    pub fn update(&mut self, c: &mut Client, net: &Connections) {
        let state = match self.frame_history.get_mut(&c.id) {
            Some(x) => x,
            None => return,
        };

        if !state.ready {
            return;
        }

        state.ready = false;

        let diff = state.inputs.len() - state.sent;

        let inputs = Vec::from(&state.inputs[state.sent..]);
        state.sent += inputs.len();

        c.ack = state.from + Frame(state.sent as u64);

        if diff <= 30 {
            log::info!("{}: sending final catch up", c.name);
            net.send_tcp(
                c.tcp_addr,
                encode(&ServerReliablePacket::ReadyToPlay {
                    final_consumed_frame: c.ack,
                    final_inputs: inputs,
                }),
            );
            c.state = ClientGameState::Playing;
            self.frame_history.remove(&c.id);
            return;
        }

        let pack = ServerReliablePacket::CatchUp { inputs };

        net.send_tcp(c.tcp_addr, encode(&pack));
    }

    pub fn disconnected(&mut self, id: AuthentID) {
        self.frame_history.remove(&id);
    }
}
