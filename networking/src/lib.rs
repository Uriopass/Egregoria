use std::ops::Add;

use bincode::Options;
use serde::{Deserialize, Serialize};

mod authent;
mod catchup;
mod client;
mod packets;
mod ring;
mod server;
mod worldsend;

pub use client::{Client, ConnectConf, PollResult};
pub use server::{Server, ServerConfiguration};

pub(crate) const MAX_CATCHUP_PACKET_SIZE: usize = 1000000; // 1000 kb ~ 125ko
pub(crate) const MAX_WORLDSEND_PACKET_SIZE: usize = 1000000; // 1000 kb ~ 125ko
pub(crate) const DEFAULT_PORT: u16 = 23019;

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash, Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Frame(pub u32);

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub(crate) struct UserID(pub u64);

#[derive(Clone, Serialize, Deserialize, Debug)]
#[repr(transparent)]
pub(crate) struct PlayerInput(pub Vec<u8>);

pub(crate) type MergedInputs = Vec<PlayerInput>;

impl Add for Frame {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Frame {
    pub fn incr(&mut self) {
        self.0 += 1
    }
}

pub(crate) fn try_encode<T: Serialize>(x: &T) -> bincode::Result<Vec<u8>> {
    bincode::DefaultOptions::new().serialize(x)
}

pub(crate) fn encode<T: Serialize>(x: &T) -> Vec<u8> {
    try_encode(x).expect("failed serializing")
}

pub(crate) fn decode<'a, T: Deserialize<'a>>(x: &'a [u8]) -> bincode::Result<T> {
    bincode::DefaultOptions::new().deserialize(x)
}
