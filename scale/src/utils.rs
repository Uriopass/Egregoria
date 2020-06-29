macro_rules! unwrap_or {
    ($e: expr, $t: expr) => {
        match $e {
            Some(x) => x,
            None => $t,
        }
    };
}

pub fn rand_world<T>(world: &mut specs::World) -> T
where
    rand_distr::Standard: rand_distr::Distribution<T>,
{
    use specs::WorldExt;
    world.write_resource::<crate::RandProvider>().random()
}

pub trait Restrict {
    fn restrict(self, min: Self, max: Self) -> Self;
}

impl<T: PartialOrd> Restrict for T {
    fn restrict(self, min: Self, max: Self) -> Self {
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }
}

pub mod debugdraw {
    use crate::geometry::Vec2;
    pub use crate::rendering::Color;
    use lazy_static::*;
    use std::sync::{Arc, Mutex};

    #[derive(Copy, Clone)]
    pub enum DebugOrder {
        Point { pos: Vec2, size: f32 },
        Line { from: Vec2, to: Vec2 },
    }

    lazy_static! {
        pub static ref DEBUG_ORDERS: Arc<Mutex<Vec<(DebugOrder, Color)>>> =
            Arc::new(Mutex::new(Vec::new()));
        pub static ref PERSISTENT_DEBUG_ORDERS: Arc<Mutex<Vec<(DebugOrder, Color)>>> =
            Arc::new(Mutex::new(Vec::new()));
    }

    pub fn debug_draw(order: DebugOrder) {
        DEBUG_ORDERS.lock().unwrap().push((order, Color::GREEN)); // Unwrap ok: Mutex lives in main thread
    }

    pub fn debug_draw_color(order: DebugOrder, color: Color) {
        DEBUG_ORDERS.lock().unwrap().push((order, color)); // Unwrap ok: Mutex lives in main thread
    }

    pub fn debug_draw_persistent(order: DebugOrder) {
        PERSISTENT_DEBUG_ORDERS
            .lock()
            .unwrap() // Unwrap ok: Mutex lives in main thread
            .push((order, Color::GREEN));
    }

    pub fn debug_draw_persistent_color(order: DebugOrder, color: Color) {
        PERSISTENT_DEBUG_ORDERS.lock().unwrap().push((order, color)); // Unwrap ok: Mutex lives in main thread
    }

    pub fn debug_clear_persistent() {
        PERSISTENT_DEBUG_ORDERS.lock().unwrap().clear(); // Unwrap ok: Mutex lives in main thread
    }
}
