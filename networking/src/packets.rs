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
    Connect { name: String },
    BeginCatchUp,
    CatchUpAck,
    WorldAck,
}

#[derive(Serialize, Deserialize)]
pub(crate) enum AuthentResponse {
    Accepted { id: AuthentID },
    Refused { reason: String },
}

#[derive(Serialize, Deserialize)]
pub(crate) struct WorldDataFragment {
    pub is_over: Option<Frame>,
    pub data: Vec<u8>,
}
