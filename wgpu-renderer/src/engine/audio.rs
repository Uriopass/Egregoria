pub struct AudioContext {
    device: Option<rodio::Device>,
    sinks: Vec<rodio::Sink>,
}

impl AudioContext {
    pub fn new(n_sinks: usize) -> Self {
        // Have to spawn a thread to initialize rodio or it'll crash
        std::thread::spawn(move || {
            let device = rodio::default_output_device();
            let mut sinks = Vec::new();
            if let Some(d) = &device {
                for _ in 0..n_sinks {
                    sinks.push(rodio::Sink::new(d));
                }
            }
            Self { device, sinks }
        })
        .join()
        .unwrap()
    }

    pub fn play_sound<S>(&self, source: S) -> bool
    where
        S: rodio::Source + Send + 'static,
        S::Item: rodio::Sample + Send,
    {
        for sink in &self.sinks {
            if sink.len() == 0 {
                sink.append(source);
                return true;
            }
        }
        false
    }
}
