//! This module contains the change detection system for the map.
//! This should not be used inside the simulation as change subscribers are not serialized.
//! It is mostly for rendering purposes by decoupling it from the simulation.

use crate::map::{chunk_id, Building, ChunkID, Intersection, Lot, Road};
use geom::Vec2;
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

// CanonicalPosition is a trait that describes the canonical position of an object.
// It is used to determine which chunk an object belongs to.
pub trait CanonicalPosition {
    fn canonical_position(&self) -> Vec2;
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum UpdateType {
    Road,
    Building,
    Terrain,
}

#[derive(Default)]
pub struct MapSubscribers(Mutex<Vec<MapSubscriber>>);

impl MapSubscribers {
    pub fn subscribe(&self, filter: UpdateType) -> MapSubscriber {
        let sub = MapSubscriber::new(filter);
        self.0.lock().unwrap().push(sub.clone());
        sub
    }

    pub fn dispatch_all(&self, chunks: impl Iterator<Item = ChunkID>) {
        let mut me = self.0.lock().unwrap();
        for chunk in chunks {
            for sub in me.iter_mut() {
                sub.dispatch(UpdateType::Road, chunk);
                sub.dispatch(UpdateType::Building, chunk);
                sub.dispatch(UpdateType::Terrain, chunk);
            }
        }
    }

    pub fn dispatch_clear(&mut self) {
        let mut me = self.0.lock().unwrap();
        for sub in me.iter_mut() {
            sub.inner.lock().unwrap().cleared = true;
        }
    }

    pub fn dispatch(&mut self, update_type: UpdateType, p: &impl CanonicalPosition) {
        let chunk_id = chunk_id(p.canonical_position());
        self.dispatch_chunk(update_type, chunk_id);
    }

    pub fn dispatch_chunk(&mut self, update_type: UpdateType, chunk_id: ChunkID) {
        let mut me = self.0.lock().unwrap();
        for sub in me.iter_mut() {
            sub.dispatch(update_type, chunk_id);
        }
    }
}

#[derive(Default)]
pub struct MapSubscriberInner {
    pub updated_chunks: BTreeSet<ChunkID>,
    pub cleared: bool,
}

/// Describes a subscriber to a specific UpdateType
#[derive(Clone)]
pub struct MapSubscriber {
    filter: UpdateType,
    inner: Arc<Mutex<MapSubscriberInner>>,
}

impl MapSubscriber {
    pub fn new(update_type: UpdateType) -> Self {
        Self {
            filter: update_type,
            inner: Default::default(),
        }
    }

    pub fn take_updated_chunks(&mut self) -> impl Iterator<Item = ChunkID> {
        let mut inner = self.inner.lock().unwrap();
        std::mem::take(&mut inner.updated_chunks).into_iter()
    }

    pub fn take_one_updated_chunk(&mut self) -> Option<ChunkID> {
        let mut inner = self.inner.lock().unwrap();
        inner.updated_chunks.pop_first()
    }

    pub fn take_cleared(&mut self) -> bool {
        let mut inner = self.inner.lock().unwrap();
        std::mem::take(&mut inner.cleared)
    }

    pub fn dispatch(&mut self, update_type: UpdateType, chunk_id: ChunkID) {
        if update_type != self.filter {
            return;
        }
        let mut inner = self.inner.lock().unwrap();
        inner.updated_chunks.insert(chunk_id);
    }
}

impl CanonicalPosition for Intersection {
    fn canonical_position(&self) -> Vec2 {
        self.pos.xy()
    }
}

impl CanonicalPosition for Road {
    fn canonical_position(&self) -> Vec2 {
        self.points.first().xy()
    }
}

impl CanonicalPosition for Building {
    fn canonical_position(&self) -> Vec2 {
        self.obb.center()
    }
}

impl CanonicalPosition for Lot {
    fn canonical_position(&self) -> Vec2 {
        self.shape.center()
    }
}
