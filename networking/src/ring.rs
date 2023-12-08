use crate::Frame;

const RING_SIZE: usize = 128;
#[derive(Debug)]
pub struct Ring<T: Default> {
    ring: [T; RING_SIZE],
}

impl<T: Default> Ring<T> {
    pub fn new() -> Self {
        Self {
            ring: [(); RING_SIZE].map(|_| Default::default()),
        }
    }

    pub fn get(&self, f: Frame) -> &T {
        &self.ring[f.0 as usize % self.ring.len()]
    }

    pub fn get_mut(&mut self, f: Frame) -> &mut T {
        &mut self.ring[f.0 as usize % self.ring.len()]
    }

    pub fn len(&self) -> usize {
        self.ring.len()
    }

    #[allow(dead_code)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.ring.iter_mut()
    }
}
