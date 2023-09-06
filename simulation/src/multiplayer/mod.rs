use crate::multiplayer::chat::Chat;
use serde::{Deserialize, Serialize};

pub mod chat;

#[derive(Default, Serialize, Deserialize)]
pub struct MultiplayerState {
    pub chat: Chat,
}
