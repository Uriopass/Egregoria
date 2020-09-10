use std::sync::{Mutex, MutexGuard};

#[derive(Default)]
pub struct FrameLog {
    logs: Mutex<Vec<String>>,
}

impl FrameLog {
    pub fn clear(&self) {
        self.logs.lock().unwrap().clear();
    }

    pub fn log_frame(&self, s: String) {
        self.logs.lock().unwrap().push(s);
    }

    pub fn get_frame_log(&self) -> MutexGuard<Vec<String>> {
        self.logs.lock().unwrap()
    }
/*
    pub fn time_guard(&self, file: &'static str, line: u32) -> LogTimeGuard {
        LogTimeGuard {
            logger: self,
            start: std::time::Instant::now(),
            file,
            line,
        }
    }*/
}

/*
macro_rules! time_it {
    ($l: expr, $s: expr) => {
        let _guard = $l.time_guard($s, 0);
    };
    ($l: expr) => {
        let _guard = $l.time_guard(file!(), line!());
    };
}

pub struct LogTimeGuard<'a> {
    logger: &'a FrameLog,
    start: std::time::Instant,
    file: &'static str,
    line: u32,
}

impl<'a> Drop for LogTimeGuard<'a> {
    fn drop(&mut self) {
        let t = self.start.elapsed().as_secs_f32() * 1000.0;
        self.logger.log_frame(if self.line > 0 {
            format!("{}:{} took {:.3}ms", self.file, self.line, t)
        } else {
            format!("{} took {:.3}ms", self.file, t)
        });
    }
}
*/
