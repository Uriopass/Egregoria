use geom::Color;
use prototypes::{GameInstant, GameTime};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Chat {
    pub messages: Vec<Message>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub name: String,
    pub text: String,
    pub sent_at: GameInstant,
    pub color: Color,
    pub kind: MessageKind,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MessageKind {
    Info,
    Warning,
    PlayerChat,
}

impl Chat {
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn messages_since(&self, time: GameInstant) -> impl Iterator<Item = &'_ Message> + '_ {
        self.messages
            .iter()
            .filter(move |m| m.sent_at >= time)
            .rev()
    }
}

impl Message {
    /// Returns the number of (game) seconds elapsed since the message was sent
    pub fn age_secs(&self, now: &GameTime) -> f64 {
        self.sent_at.elapsed(now).seconds()
    }
}
