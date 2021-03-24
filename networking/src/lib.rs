use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::ops::Add;

mod authent;
mod catchup;
mod client;
mod packets;
mod ring;
mod server;
mod worldsend;

pub use client::{Client, ConnectConf, PollResult};
use common::saveload::{CompressedBincode, Encoder};
use serde::de::DeserializeOwned;
pub use server::{Server, ServerConfiguration};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub(crate) const MAX_CATCHUP_PACKET_SIZE: usize = 1000000; // 1000 kb ~ 125ko
pub(crate) const MAX_WORLDSEND_PACKET_SIZE: usize = 1000000; // 1000 kb ~ 125ko
pub(crate) const DEFAULT_PORT: u16 = 23019;

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash, Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Frame(pub u32);

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub(crate) struct UserID(pub u32);

#[derive(Clone, Serialize, Deserialize, Debug)]
#[repr(transparent)]
pub(crate) struct PlayerInput(pub Vec<u8>);

pub(crate) type MergedInputs = Vec<(UserID, PlayerInput)>;

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
    pub fn decr(&mut self) {
        self.0 -= 1
    }
}

type Enc = CompressedBincode;

pub(crate) fn try_encode<T: Serialize>(x: &T) -> Option<Vec<u8>> {
    Enc::encode(x).ok()
}

pub(crate) fn encode<T: Serialize>(x: &T) -> Vec<u8> {
    try_encode(x).expect("failed serializing")
}

pub(crate) fn decode<T: DeserializeOwned>(x: &[u8]) -> Option<T> {
    Enc::decode(x).ok()
}

pub(crate) struct PhantomSendSync<T>(PhantomData<T>);

unsafe impl<T> Send for PhantomSendSync<T> {}
unsafe impl<T> Sync for PhantomSendSync<T> {}

impl<T> Default for PhantomSendSync<T> {
    fn default() -> Self {
        Self(PhantomData::default())
    }
}

pub(crate) fn hash_str(s: &str) -> u32 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish() as u32
}
