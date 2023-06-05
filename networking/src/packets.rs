use crate::authent::AuthentID;
use crate::{Frame, MergedInputs, PlayerInput};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ServerUnreliablePacket {
    Input(Vec<(Frame, MergedInputs)>),
    ReadyForAuth,
}

#[derive(Serialize, Deserialize)]
pub(crate) enum ServerReliablePacket {
    Challenge(AuthentID),
    ReadyToPlay {
        final_consumed_frame: Frame,
        final_inputs: Vec<MergedInputs>,
    },
    AuthentResponse(AuthentResponse),
    CatchUp {
        inputs: Vec<MergedInputs>,
    },
    WorldSend(WorldDataFragment),
}

#[derive(Serialize, Deserialize)]
pub(crate) enum ClientUnreliablePacket {
    Connection(AuthentID),
    Input { input: Vec<(Frame, PlayerInput)> },
}

#[derive(Serialize, Deserialize)]
pub(crate) enum ClientReliablePacket {
    Connect { name: String, version: String },
    BeginCatchUp,
    CatchUpAck,
    WorldAck,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) enum AuthentResponse {
    Accepted { id: AuthentID, period: Duration },
    Refused { reason: String },
}

#[derive(Serialize, Deserialize)]
pub(crate) struct WorldDataFragment {
    pub is_over: Option<Frame>,
    pub data_size: usize,
    pub data: Vec<u8>,
}
