use rodio::{OutputStreamHandle, PlayError, Sample, Source};
use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::Arc;
use std::time::Duration;

/// Copy of rodio::sink with some modifications
pub struct UniqueSink {
    controls: Arc<Controls>,
}

struct Controls {
    volume: AtomicU32,
    stopped: AtomicBool,
    dead: AtomicBool,
}

#[allow(dead_code)]
impl UniqueSink {
    /// Builds a new `UniqueSink`, beginning playback on a stream.
    #[inline]
    pub fn try_new<S>(stream: &OutputStreamHandle, source: S) -> Result<UniqueSink, PlayError>
    where
        S: Source + Send + 'static,
        S::Item: Sample,
        S::Item: Send,
    {
        let controls = Arc::new(Controls {
            volume: 1.0f32.to_bits().into(),
            stopped: AtomicBool::new(false),
            dead: AtomicBool::new(false),
        });

        let unique_sink = UniqueSink {
            controls: controls.clone(),
        };

        let source = SinkSource {
            source: source.convert_samples(),
            cur_volume: 0.0,
            controls,
            period: 50,
            remaining: 50,
        };

        stream.play_raw(source)?;
        Ok(unique_sink)
    }

    pub fn is_dead(&self) -> bool {
        self.controls.dead.load(Ordering::SeqCst)
    }

    /// Gets the volume of the sound.
    ///
    /// The value `1.0` is the "normal" volume (unfiltered input). Any value other than 1.0 will
    /// multiply each sample by this value.
    #[inline]
    pub fn volume(&self) -> f32 {
        f32::from_bits(self.controls.volume.load(Ordering::SeqCst))
    }

    /// Changes the volume of the sound.
    ///
    /// The value `1.0` is the "normal" volume (unfiltered input). Any value other than `1.0` will
    /// multiply each sample by this value.
    #[inline]
    pub fn set_volume(&self, value: f32) {
        self.controls
            .volume
            .store(value.to_bits(), Ordering::SeqCst);
    }

    /// Stops the uniqueSink by emptying the queue.
    #[inline]
    pub fn stop(&self) {
        self.controls.stopped.store(true, Ordering::SeqCst);
    }
}

struct SinkSource<S: Source<Item = f32>> {
    source: S,
    controls: Arc<Controls>,
    cur_volume: f32,
    period: u32,
    remaining: u32,
}

impl<S: Source<Item = f32>> Iterator for SinkSource<S> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining > 0 {
            self.remaining -= 1;
            return if let Some(x) = self.source.next() {
                Some(x * self.cur_volume)
            } else {
                self.controls.dead.store(true, Ordering::SeqCst);
                None
            };
        }

        self.remaining = self.period;
        let controls = &*self.controls;
        if controls.stopped.load(Ordering::SeqCst) {
            None
        } else {
            let v = f32::from_bits(controls.volume.load(Ordering::SeqCst));
            self.cur_volume = self.cur_volume + (v - self.cur_volume) * 0.01;
            if let Some(x) = self.source.next() {
                Some(x * self.cur_volume)
            } else {
                self.controls.dead.store(true, Ordering::SeqCst);
                None
            }
        }
    }
}

impl<S: Source<Item = f32>> Source for SinkSource<S> {
    fn current_frame_len(&self) -> Option<usize> {
        self.source.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.source.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Drop for UniqueSink {
    #[inline]
    fn drop(&mut self) {
        self.controls.stopped.store(true, Ordering::Relaxed);
    }
}
