use log::{Level, LevelFilter, Metadata, Record};
use std::fs::File;
use std::io::{stdout, BufWriter, Write};
use std::sync::Mutex;
use std::time::Instant;

/// A logger that logs to a file and stdout
pub struct MyLog {
    start: Instant,
    log_file: Option<Mutex<BufWriter<File>>>,
}

impl MyLog {
    fn new() -> Self {
        let log_file;
        #[cfg(not(debug_assertions))]
        {
            let _ = std::fs::create_dir("logs");
            use std::time::SystemTime;
            log_file = File::create(format!(
                "logs/log_{}.log",
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("what is this, an IBM mainframe?")
                    .as_micros()
            ))
            .ok()
            .map(|f| Mutex::new(BufWriter::new(f)));
        }

        #[cfg(any(debug_assertions, test))]
        {
            log_file = None;
        }

        Self {
            start: Instant::now(),
            log_file,
        }
    }

    pub fn init() {
        let leaked = Box::leak(Box::new(MyLog::new()));
        crate::unwrap_orr!(log::set_logger(leaked), return);
        log::set_max_level(LevelFilter::Debug);
        log_panics::init();
    }
}

macro_rules! write_log_stdout {
    ($file:expr, $($arg:tt)*) => {
        let _ = println!($($arg)*);

        if let Some(ref m) = $file {
            let mut bw = m.lock().unwrap();
            let _ = writeln!(bw, $($arg)*);
            let _ = bw.flush();
        }
    }
}

impl log::Log for MyLog {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        let l = metadata.level();
        match metadata.target() {
            "gpu_alloc::buddy"
            | "gpu_alloc::linear"
            | "gpu_alloc::allocator"
            | "tracing::span"
            | "wgpu_core"
            | "wgpu_core::device"
            | "wgpu_core::hub"
            | "wgpu_core::device::global"
            | "wgpu_hal::vulkan::instance"
            | "wgpu_hal::dx12::device"
            | "gfx_memory"
            | "gfx_backend_vulkan"
            | "gfx_descriptor" => l <= Level::Warn,
            _ => true,
        }
    }

    fn log(&self, r: &Record<'_>) {
        if r.target() == "panic" {
            write_log_stdout!(self.log_file, "{}", r.args());
            self.flush();
            return;
        }

        if std::thread::panicking() {
            self.flush();
            return;
        }

        if !self.enabled(r.metadata()) {
            return;
        }

        let time = self.start.elapsed().as_micros();
        if r.level() > Level::Warn {
            let module_path = r
                .module_path_static()
                //.and_then(|x| x.split(':').last())
                .unwrap_or_else(|| r.target());
            write_log_stdout!(
                self.log_file,
                "[{:9} {:5} {:12}] {}",
                time,
                r.level(),
                module_path,
                r.args()
            );
        } else {
            write_log_stdout!(
                self.log_file,
                "[{:9} {:5} {}:{}] {}",
                time,
                r.level(),
                r.file().unwrap_or_default(),
                r.line().unwrap_or_default(),
                r.args()
            );
        }
    }

    fn flush(&self) {
        let _ = stdout().flush();
        if let Some(ref x) = self.log_file {
            let _ = x.lock().unwrap().flush();
        }
    }
}
