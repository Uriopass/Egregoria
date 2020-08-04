use lazy_static::*;
use std::sync::{Mutex, MutexGuard};

lazy_static! {
    static ref LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

pub fn clear() {
    LOG.lock().unwrap().clear();
}

pub fn log_frame(s: String) {
    LOG.lock().unwrap().push(s);
}

pub fn get_frame_log<'a>() -> MutexGuard<'a, Vec<String>> {
    LOG.lock().unwrap()
}

macro_rules! time_it {
    ($s: expr) => {
        let _guard = crate::log::LogTimeGuard::new($s, 0);
    };
    () => {
        let _guard = crate::log::LogTimeGuard::new(file!(), line!());
    };
}

pub struct LogTimeGuard {
    start: std::time::Instant,
    file: &'static str,
    line: u32,
}

impl LogTimeGuard {
    pub fn new(file: &'static str, line: u32) -> Self {
        Self {
            start: std::time::Instant::now(),
            file,
            line,
        }
    }
}

impl Drop for LogTimeGuard {
    fn drop(&mut self) {
        let t = self.start.elapsed().as_secs_f32() * 1000.0;
        log_frame(if self.line > 0 {
            format!("{}:{} took {:.3}ms", self.file, self.line, t)
        } else {
            format!("{} took {:.3}ms", self.file, t)
        });
    }
}
