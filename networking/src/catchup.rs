use crate::authent::Client;
use crate::packets::ServerReliablePacket;
use crate::{encode, Frame, MergedInputs, UserID, MAX_CATCHUP_PACKET_SIZE};
use message_io::network::Network;
use std::collections::HashMap;

struct CatchUpState {
    inputs: Vec<MergedInputs>,
    sent: usize,
    from: Frame,
    ready: bool,
}

#[derive(Default)]
pub(crate) struct CatchUp {
    frame_history: HashMap<UserID, CatchUpState>,
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
            if frame.0 != v.from.0 + 1 + v.inputs.len() as u32 {
                log::error!("wrong input for catch up !!!")
            }
            v.inputs.push(inp.clone())
        }
    }

    pub fn ack(&mut self, c: &Client) {
        self.frame_history.get_mut(&c.id).unwrap().ready = true;
    }

    pub fn update(&mut self, c: &mut Client, net: &mut Network) {
        let state = match self.frame_history.get_mut(&c.id) {
            Some(x) => x,
            None => return,
        };

        if !state.ready {
            return;
        }

        state.ready = false;

        let diff = state.inputs.len() - state.sent;

        if diff <= 30 {
            log::info!("{}: sending ready to play", c.name);
            net.send(
                c.reliable,
                &*encode(&ServerReliablePacket::ReadyToPlay { start_frame: c.ack }),
            );
            self.frame_history.remove(&c.id);
            return;
        }

        let mut inputs = vec![];
        let mut size = 0;
        while size < MAX_CATCHUP_PACKET_SIZE && state.sent < state.inputs.len() {
            let d = state.inputs[state.sent].clone();
            size += d.iter().map(|x| x.0.len()).sum::<usize>();
            inputs.push(d);
            state.sent += 1;
        }

        let pack = ServerReliablePacket::CatchUp { inputs };

        net.send(c.reliable, &*encode(&pack));

        c.ack = state.from + Frame(state.sent as u32);
    }

    pub fn disconnected(&mut self, id: UserID) {
        self.frame_history.remove(&id);
    }
}
