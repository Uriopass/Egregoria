use std::sync::atomic::AtomicUsize;

#[derive(Default)]
pub struct PerfCounters {
    total_triangles: AtomicUsize,
    total_drawcalls: AtomicUsize,

    depth_triangles: AtomicUsize,
    depth_drawcalls: AtomicUsize,

    shadows_triangles: AtomicUsize,
    shadows_drawcalls: AtomicUsize,

    heightmap_triangles: AtomicUsize,
    heightmap_depth_triangles: AtomicUsize,
    heightmap_shadows_triangles: AtomicUsize,
}

pub struct PerfCountersStatic {
    pub total_triangles: usize,
    pub total_drawcalls: usize,

    pub depth_triangles: usize,
    pub depth_drawcalls: usize,

    pub shadows_triangles: usize,
    pub shadows_drawcalls: usize,

    pub heightmap_triangles: usize,
    pub heightmap_depth_triangles: usize,
    pub heightmap_shadows_triangles: usize,
}

impl PerfCounters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_static(&mut self) -> PerfCountersStatic {
        PerfCountersStatic {
            total_triangles: *self.total_triangles.get_mut(),
            total_drawcalls: *self.total_drawcalls.get_mut(),
            depth_triangles: *self.depth_triangles.get_mut(),
            depth_drawcalls: *self.depth_drawcalls.get_mut(),
            shadows_triangles: *self.shadows_triangles.get_mut(),
            shadows_drawcalls: *self.shadows_drawcalls.get_mut(),
            heightmap_triangles: *self.heightmap_triangles.get_mut(),
            heightmap_depth_triangles: *self.heightmap_depth_triangles.get_mut(),
            heightmap_shadows_triangles: *self.heightmap_shadows_triangles.get_mut(),
        }
    }

    pub fn clear(&mut self) {
        *self.total_triangles.get_mut() = 0;
        *self.total_drawcalls.get_mut() = 0;
        *self.depth_triangles.get_mut() = 0;
        *self.depth_drawcalls.get_mut() = 0;
        *self.shadows_triangles.get_mut() = 0;
        *self.shadows_drawcalls.get_mut() = 0;
        *self.heightmap_triangles.get_mut() = 0;
        *self.heightmap_depth_triangles.get_mut() = 0;
        *self.heightmap_shadows_triangles.get_mut() = 0;
    }

    pub fn drawcall(&self, triangles: impl TryInto<usize>) {
        self.total_triangles.fetch_add(
            triangles.try_into().unwrap_or(0),
            std::sync::atomic::Ordering::Relaxed,
        );
        self.total_drawcalls
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn depth_drawcall(&self, triangles: impl TryInto<usize>, shadow: bool) {
        if shadow {
            self.shadows_triangles.fetch_add(
                triangles.try_into().unwrap_or(0),
                std::sync::atomic::Ordering::Relaxed,
            );
            self.shadows_drawcalls
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return;
        }
        self.depth_triangles.fetch_add(
            triangles.try_into().unwrap_or(0),
            std::sync::atomic::Ordering::Relaxed,
        );
        self.depth_drawcalls
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn heightmap_drawcall(&self, triangles: impl TryInto<usize>) {
        self.heightmap_triangles.fetch_add(
            triangles.try_into().unwrap_or(0),
            std::sync::atomic::Ordering::Relaxed,
        );
    }

    pub fn heightmap_depth_drawcall(&self, triangles: impl TryInto<usize>, shadows: bool) {
        if shadows {
            self.heightmap_shadows_triangles.fetch_add(
                triangles.try_into().unwrap_or(0),
                std::sync::atomic::Ordering::Relaxed,
            );
            return;
        }
        self.heightmap_depth_triangles.fetch_add(
            triangles.try_into().unwrap_or(0),
            std::sync::atomic::Ordering::Relaxed,
        );
    }
}
