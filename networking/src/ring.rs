use crate::Frame;

const RING_SIZE: usize = 128;
pub struct Ring<T: Default> {
    ring: [T; RING_SIZE],
}

impl<T: Default> Ring<T> {
    pub fn new() -> Self {
        Self { ring: arr_init() }
    }

    pub fn get(&self, f: Frame) -> &T {
        &self.ring[f.0 as usize % self.ring.len()]
    }

    pub fn get_mut(&mut self, f: Frame) -> &mut T {
        &mut self.ring[f.0 as usize % self.ring.len()]
    }

    pub fn len(&self) -> u32 {
        self.ring.len() as u32
    }

    #[allow(dead_code)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.ring.iter_mut()
    }
}

fn arr_init<T: Default>() -> [T; RING_SIZE] {
    let mut data: [T; RING_SIZE] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
    for i in 0..RING_SIZE {
        unsafe {
            data.as_mut_ptr().add(i).write(T::default());
        }
    }
    data
}
