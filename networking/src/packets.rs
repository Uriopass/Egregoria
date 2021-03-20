use crate::authent::AuthentID;
use crate::{Frame, MergedInputs, PlayerInput};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) enum ServerUnreliablePacket {
    Input(Vec<(Frame, MergedInputs)>),
}

#[derive(Serialize, Deserialize)]
pub(crate) enum ServerReliablePacket {
    Challenge(AuthentID),
    ReadyForAuth,
    ReadyToPlay { start_frame: Frame },
    AuthentResponse(AuthentResponse),
    CatchUp { inputs: Vec<MergedInputs> },
    WorldSend(WorldDataFragment),
}

#[derive(Serialize, Deserialize)]
pub(crate) enum ClientUnreliablePacket {
    Connection(AuthentID),
    Input {
        input: Vec<(Frame, PlayerInput)>,
        ack_frame: Frame,
    },
}

#[derive(Serialize, Deserialize)]
pub(crate) enum ClientReliablePacket {
    Connect { name: String },
    BeginCatchUp,
    CatchUpAck,
    ReadyToPlayAck,
    WorldAck,
}

#[derive(Serialize, Deserialize)]
pub(crate) enum AuthentResponse {
    Accepted,
    Refused { reason: String },
}

#[derive(Serialize, Deserialize)]
pub(crate) struct WorldDataFragment {
    pub is_over: bool,
    pub data: Vec<u8>,
}
