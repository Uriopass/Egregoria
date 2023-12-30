use std::sync::atomic::AtomicUsize;

#[derive(Default)]
pub struct PerfCounters {
    total_triangles: AtomicUsize,
    total_drawcalls: AtomicUsize,

    depth_triangles: AtomicUsize,
    depth_drawcalls: AtomicUsize,

    shadows_triangles: AtomicUsize,
    shadows_drawcalls: AtomicUsize,

    terrain_triangles: AtomicUsize,
    terrain_depth_triangles: AtomicUsize,
    terrain_shadows_triangles: AtomicUsize,
}

pub struct PerfCountersStatic {
    pub total_triangles: usize,
    pub total_drawcalls: usize,

    pub depth_triangles: usize,
    pub depth_drawcalls: usize,

    pub shadows_triangles: usize,
    pub shadows_drawcalls: usize,

    pub terrain_triangles: usize,
    pub terrain_depth_triangles: usize,
    pub terrain_shadows_triangles: usize,
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
            terrain_triangles: *self.terrain_triangles.get_mut(),
            terrain_depth_triangles: *self.terrain_depth_triangles.get_mut(),
            terrain_shadows_triangles: *self.terrain_shadows_triangles.get_mut(),
        }
    }

    pub fn clear(&mut self) {
        *self.total_triangles.get_mut() = 0;
        *self.total_drawcalls.get_mut() = 0;
        *self.depth_triangles.get_mut() = 0;
        *self.depth_drawcalls.get_mut() = 0;
        *self.shadows_triangles.get_mut() = 0;
        *self.shadows_drawcalls.get_mut() = 0;
        *self.terrain_triangles.get_mut() = 0;
        *self.terrain_depth_triangles.get_mut() = 0;
        *self.terrain_shadows_triangles.get_mut() = 0;
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

    pub fn terrain_drawcall(&self, triangles: impl TryInto<usize>) {
        self.terrain_triangles.fetch_add(
            triangles.try_into().unwrap_or(0),
            std::sync::atomic::Ordering::Relaxed,
        );
    }

    pub fn terrain_depth_drawcall(&self, triangles: impl TryInto<usize>, shadows: bool) {
        if shadows {
            self.terrain_shadows_triangles.fetch_add(
                triangles.try_into().unwrap_or(0),
                std::sync::atomic::Ordering::Relaxed,
            );
            return;
        }
        self.terrain_depth_triangles.fetch_add(
            triangles.try_into().unwrap_or(0),
            std::sync::atomic::Ordering::Relaxed,
        );
    }
}
