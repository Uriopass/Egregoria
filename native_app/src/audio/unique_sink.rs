use rodio::{OutputStreamHandle, PlayError, Sample, Source};
use std::iter::Peekable;
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
    speed: AtomicU32,
    dead: AtomicBool,
}

#[allow(dead_code)]
impl UniqueSink {
    /// Builds a new `UniqueSink`, beginning playback on a stream.
    #[inline]
    pub fn try_new<S>(
        stream: &OutputStreamHandle,
        source: S,
        complex: bool,
    ) -> Result<UniqueSink, PlayError>
    where
        S: Source + Send + 'static,
        S::Item: Sample,
        S::Item: Send,
    {
        let controls = Arc::new(Controls {
            volume: 1.0f32.to_bits().into(),
            speed: 0.01f32.to_bits().into(),
            dead: AtomicBool::new(false),
        });

        let unique_sink = UniqueSink {
            controls: controls.clone(),
        };

        if complex {
            stream.play_raw(ComplexSinkSource::new(source.convert_samples(), controls))?;
        } else {
            stream.play_raw(SimpleSinkSource::new(source.convert_samples(), controls))?;
        }

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

    #[inline]
    pub fn set_speed(&self, speed: f32) {
        self.controls.speed.store(speed.to_bits(), Ordering::SeqCst);
    }
}

struct SimpleSinkSource<S: Source<Item = f32>> {
    source: S,
    controls: Arc<Controls>,
    volume: f32,

    period: u32,
    remaining: u32,
}

impl<S: Source<Item = f32>> SimpleSinkSource<S> {
    pub fn new(mut source: S, controls: Arc<Controls>) -> Self {
        Self {
            source,
            volume: f32::from_bits(controls.volume.load(Ordering::SeqCst)),
            controls,
            period: 50,
            remaining: 50,
        }
    }
}

impl<S: Source<Item = f32>> Iterator for SimpleSinkSource<S> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let v = if let Some(x) = self.source.next() {
            x
        } else {
            self.controls.dead.store(true, Ordering::SeqCst);
            return None;
        };

        if self.remaining == 0 {
            self.remaining = self.period + 1;

            let controls = &*self.controls;

            let v = f32::from_bits(controls.volume.load(Ordering::SeqCst));
            self.volume += (v - self.volume) * 0.01;
        }
        self.remaining -= 1;

        Some(v * self.volume)
    }
}

struct ComplexSinkSource<S: Source<Item = f32>> {
    source: S,
    controls: Arc<Controls>,
    volume: f32,

    speed: f32,
    remainder: f32,

    sample: [f32; 2],
    peek: [f32; 2],
    channel_id: usize,

    period: u32,
    remaining: u32,
}

impl<S: Source<Item = f32>> ComplexSinkSource<S> {
    pub fn new(mut source: S, controls: Arc<Controls>) -> Self {
        let s1 = source.next().unwrap_or(0.0);
        let s2 = source.next().unwrap_or(0.0);
        let p1 = source.next().unwrap_or(0.0);
        let p2 = source.next().unwrap_or(0.0);
        Self {
            sample: [s1, s2],
            peek: [p1, p2],
            channel_id: 0,
            source,
            volume: f32::from_bits(controls.volume.load(Ordering::SeqCst)),
            speed: f32::from_bits(controls.speed.load(Ordering::SeqCst)),
            remainder: 0.0,
            controls,
            period: 50,
            remaining: 50,
        }
    }
}

impl<S: Source<Item = f32>> Iterator for ComplexSinkSource<S> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let interp = (unsafe { self.sample.get_unchecked(self.channel_id) }
            * (1.0 - self.remainder)
            + self.remainder * unsafe { self.peek.get_unchecked(self.channel_id) })
            * self.volume;

        if self.channel_id == 0 {
            self.channel_id = 1;
            return Some(interp);
        }
        self.channel_id = 0;

        self.remainder += self.speed;
        while self.remainder >= 1.0 {
            self.remainder -= 1.0;
            self.sample = self.peek;

            if let Some((peek1, peek2)) = self.source.next().zip(self.source.next()) {
                self.peek = [peek1, peek2];
            } else {
                self.controls.dead.store(true, Ordering::SeqCst);
                return None;
            }
        }

        if self.remaining == 0 {
            self.remaining = self.period + 1;

            let controls = &*self.controls;
            let v = f32::from_bits(controls.volume.load(Ordering::SeqCst));
            self.volume += (v - self.volume) * 0.01;

            let v = f32::from_bits(controls.speed.load(Ordering::SeqCst));
            self.speed = v; // (v - self.cur_speed) * 0.01;
        }
        self.remaining -= 1;

        Some(interp)
    }
}

impl<S: Source<Item = f32>> Source for ComplexSinkSource<S> {
    fn current_frame_len(&self) -> Option<usize> {
        None
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

impl<S: Source<Item = f32>> Source for SimpleSinkSource<S> {
    fn current_frame_len(&self) -> Option<usize> {
        None
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
