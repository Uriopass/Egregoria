use common::{FastMap, FastSet};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use oddio::{
    FixedGain, Frame, Frames, FramesSignal, Mixed, Mixer, MixerControl, Sample, Signal, Smoothed,
};
use std::cell::RefCell;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

type StoredAudio = Arc<Frames<[Sample; 2]>>;

// We allow dead_code because we need to keep OutputStream alive for it to work
#[allow(dead_code)]
pub struct AudioContext {
    stream: Option<cpal::Stream>,
    scene_handle: Option<MixerControl<[Sample; 2]>>,
    cache: Arc<RwLock<FastMap<String, StoredAudio>>>,
    preloading: FastSet<String>,
}

#[derive(Copy, Clone)]
pub enum AudioKind {
    Music,
    Effect,
    Ui,
}

static MASTER_SHARED: AtomicU32 = AtomicU32::new(0);
static MUSIC_SHARED: AtomicU32 = AtomicU32::new(0);
static EFFECT_SHARED: AtomicU32 = AtomicU32::new(0);
static UI_SHARED: AtomicU32 = AtomicU32::new(0);

pub type Stereo = [Sample; 2];
pub type BaseSignal = FramesSignal<Stereo>;

impl AudioContext {
    pub fn empty<T: Debug>(x: T) -> Self {
        log::error!("Couldn't initialize audio because: {:?}", x);
        Self {
            stream: None,
            scene_handle: None,
            cache: Default::default(),
            preloading: Default::default(),
        }
    }
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(x) => x,
            None => return Self::empty("no output device found"),
        };
        let sample_rate = match device.default_output_config() {
            Ok(x) => x,
            Err(e) => return Self::empty(e),
        }
        .sample_rate();

        let config = cpal::StreamConfig {
            channels: 2,
            sample_rate,
            buffer_size: cpal::BufferSize::Fixed(4096), // Using BufferSize::Default causes 100% cpu usage with ALSA on linux
        };

        let (scene_handle, mut scene) = Mixer::new();

        let build_result = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let frames = oddio::frame_stereo(data);
                oddio::run(&mut scene, sample_rate.0, frames);
            },
            move |err| {
                eprintln!("{err:?}");
            },
            Some(Duration::from_secs(1)),
        );
        let stream = match build_result {
            Ok(x) => x,
            Err(e) => return Self::empty(e),
        };
        match stream.play() {
            Ok(_) => {}
            Err(e) => return Self::empty(e),
        };

        Self {
            stream: Some(stream),
            scene_handle: Some(scene_handle),
            cache: Default::default(),
            preloading: Default::default(),
        }
    }

    pub fn preload<'a>(&mut self, sounds: impl Iterator<Item = &'a str> + Send + 'static) {
        sounds.for_each(move |v| {
            self.preloading.insert(v.to_string());
            let s = v.to_string();
            let cache = self.cache.clone();
            rayon::spawn(move || {
                if let Some(audio) = Self::decode(&s) {
                    cache.write().unwrap().insert(s, audio);
                }
            });
        });
    }

    pub fn g_volume(&self, kind: AudioKind) -> f32 {
        let master = f32::from_bits(MASTER_SHARED.load(Ordering::Relaxed));
        master
            * match kind {
                AudioKind::Music => f32::from_bits(MUSIC_SHARED.load(Ordering::Relaxed)),
                AudioKind::Effect => f32::from_bits(EFFECT_SHARED.load(Ordering::Relaxed)),
                AudioKind::Ui => f32::from_bits(UI_SHARED.load(Ordering::Relaxed)),
            }
    }

    fn decode(name: &str) -> Option<StoredAudio> {
        #[cfg(debug_assertions)]
        if name.starts_with("music") {
            return None;
        }
        let p = format!("assets/sounds/{name}.ogg");
        let t = Instant::now();
        let buf = match common::saveload::load_raw(&p) {
            Ok(x) => x,
            Err(e) => {
                log::error!("Could not load sound {}: {}", name, e);
                return None;
            }
        };

        let mut decoder =
            lewton::inside_ogg::OggStreamReader::new(std::io::Cursor::new(buf)).ok()?;

        let mut samples = vec![];
        let mono = decoder.ident_hdr.audio_channels == 1;

        while let Some(packets) = decoder.read_dec_packet().expect("error decoding") {
            if mono {
                let mut it = packets.into_iter();
                let center = it.next()?;
                samples.extend(center.into_iter().map(|x| {
                    let v = x as f32 / (i16::MAX as f32);
                    [v, v]
                }))
            } else {
                let mut it = packets.into_iter();
                let left = it.next()?;
                let right = it.next()?;
                samples.extend(
                    left.into_iter()
                        .zip(right)
                        .map(|(x, y)| [x as f32 / (i16::MAX as f32), y as f32 / (i16::MAX as f32)]),
                )
            }
        }

        let frames = Frames::from_slice(decoder.ident_hdr.audio_sample_rate, &samples);

        log::info!(
            "decoding {}: {} sps|{} total samples|{} channels|took {}ms",
            &p,
            decoder.ident_hdr.audio_sample_rate,
            samples.len(),
            decoder.ident_hdr.audio_channels,
            1000.0 * t.elapsed().as_secs_f32()
        );

        Some(frames)
    }

    fn get(
        preloading: &FastSet<String>,
        cache: &RwLock<FastMap<String, StoredAudio>>,
        name: &str,
    ) -> Option<StoredAudio> {
        if preloading.contains(name) {
            for _ in 0..100 {
                if let Some(v) = cache.read().unwrap().get(name) {
                    return Some(v.clone());
                }
                std::thread::sleep(Duration::from_millis(100))
            }
        }
        if let Some(v) = cache.read().unwrap().get(name) {
            return Some(v.clone());
        }
        if let Some(decoded) = Self::decode(name) {
            cache
                .write()
                .unwrap()
                .insert(name.to_string(), decoded.clone());
            return Some(decoded);
        }
        None
    }

    pub fn play(&mut self, name: &'static str, kind: AudioKind) {
        let vol = self.g_volume(kind);
        if let Some(ref mut h) = self.scene_handle {
            if let Some(x) = Self::get(&self.preloading, &self.cache, name) {
                if let AudioKind::Music = kind {
                    log::error!("shouldn't play music with base play as it's not affected by global volume changes");
                }
                if vol <= 0.001 {
                    return;
                }
                let g = FixedGain::new(FramesSignal::new(x, 0.0).1, vol.log10() * 20.0);
                h.play(g);
            }
        }
    }

    pub fn is_all_ready(&self) -> bool {
        self.cache.read().unwrap().len() >= self.preloading.len()
    }

    pub fn play_with_control<S: 'static, Control>(
        &mut self,
        name: &'static str,
        transform: impl FnOnce(StoredAudio) -> (Control, S),
        kind: AudioKind,
    ) -> Option<(Control, Mixed)>
    where
        S: Signal<Frame = [Sample; 2]> + Send,
    {
        if let Some(ref mut h) = self.scene_handle {
            if let Some(x) = Self::get(&self.preloading, &self.cache, name) {
                let (control, signal) = transform(x);
                let test = GlobalGain {
                    volume: RefCell::new(Smoothed::new(1.0)),
                    kind,
                    inner: signal,
                };
                let mixed = h.play(test);
                return Some((control, mixed));
            }
        }
        None
    }

    pub fn set_settings(
        &mut self,
        master_volume_percent: f32,
        ui_volume_percent: f32,
        music_volume_percent: f32,
        effects_volume_percent: f32,
    ) {
        let master_volume = (master_volume_percent / 100.0).powi(2);
        if (f32::from_bits(MASTER_SHARED.load(Ordering::Relaxed)) - master_volume).abs()
            > f32::EPSILON
        {
            MASTER_SHARED.store(master_volume.to_bits(), Ordering::Relaxed);
        }

        let ui_volume = (ui_volume_percent / 100.0).powi(2);
        if (f32::from_bits(UI_SHARED.load(Ordering::Relaxed)) - ui_volume).abs() > f32::EPSILON {
            UI_SHARED.store(ui_volume.to_bits(), Ordering::Relaxed);
        }

        let music_volume = (music_volume_percent / 100.0).powi(2);
        if (f32::from_bits(MUSIC_SHARED.load(Ordering::Relaxed)) - music_volume).abs()
            > f32::EPSILON
        {
            MUSIC_SHARED.store(music_volume.to_bits(), Ordering::Relaxed);
        }

        let effect_volume = (effects_volume_percent / 100.0).powi(2);
        if (f32::from_bits(EFFECT_SHARED.load(Ordering::Relaxed)) - effect_volume).abs()
            > f32::EPSILON
        {
            EFFECT_SHARED.store(effect_volume.to_bits(), Ordering::Relaxed);
        }
    }
}

pub struct GlobalGain<T: ?Sized> {
    volume: RefCell<Smoothed<f32>>,
    kind: AudioKind,
    inner: T,
}

impl<T: Signal<Frame = [Sample; 2]>> Signal for GlobalGain<T> {
    type Frame = [Sample; 2];

    fn sample(&mut self, interval: f32, out: &mut [Self::Frame]) {
        fn upd(x: &AtomicU32, gain: &mut std::cell::RefMut<Smoothed<f32>>) {
            let master = f32::from_bits(MASTER_SHARED.load(Ordering::Relaxed));
            let shared = master * f32::from_bits(x.load(Ordering::Relaxed));
            if gain.get() != shared {
                gain.set(shared);
            }
        }

        let mut gain = self.volume.borrow_mut();
        match self.kind {
            AudioKind::Music => {
                upd(&MUSIC_SHARED, &mut gain);
            }
            AudioKind::Effect => {
                upd(&EFFECT_SHARED, &mut gain);
            }
            AudioKind::Ui => {
                upd(&UI_SHARED, &mut gain);
            }
        };

        if gain.get() == 0.0 {
            out.fill([0.0; 2]);
            return;
        }

        self.inner.sample(interval, out);
        for x in out {
            let g = gain.get();
            x[0] *= g;
            x[1] *= g;
            gain.advance(interval * 30.0);
        }
    }
}

pub struct FadeIn<T: ?Sized> {
    fadetime: f32,
    advance: RefCell<f32>,
    inner: T,
}

impl<T> FadeIn<T> {
    pub fn new(signal: T, fadetime: f32) -> Self {
        Self {
            fadetime,
            advance: RefCell::new(0.0),
            inner: signal,
        }
    }
}

impl<T: Signal<Frame = [Sample; 2]>> Signal for FadeIn<T> {
    type Frame = [Sample; 2];

    fn sample(&mut self, interval: f32, out: &mut [Self::Frame]) {
        self.inner.sample(interval, out);

        let mut advance = self.advance.borrow_mut();
        if *advance >= 1.0 {
            return;
        }

        for x in out {
            x[0] *= *advance;
            x[1] *= *advance;
            *advance += interval / self.fadetime;
        }
    }
}

/// Amplifies a signal dynamically
///
/// To implement a volume control, place a gain combinator near the end of your pipeline where the
/// input amplitude is initially in the range [0, 1] and pass decibels to [`oddio::GainControl::set_gain`],
/// mapping the maximum volume to 0 decibels, and the minimum to e.g. -60.
///
/// Forked from oddio to allow directly setting the volume to avoid pops at zero volume
/// due to Ordering::Relaxed being too relaxed
pub struct Gain<T: ?Sized> {
    shared: Arc<AtomicU32>,
    gain: Smoothed<f32>,
    inner: T,
}

impl<T> Gain<T> {
    /// Apply dynamic amplification to `signal`
    pub fn new(signal: T, vol: f32) -> (GainControl, Self) {
        let signal = Gain {
            shared: Arc::new(AtomicU32::new(vol.to_bits())),
            gain: Smoothed::new(vol),
            inner: signal,
        };
        let handle = GainControl(signal.shared.clone());
        (handle, signal)
    }
}

impl<T: Signal> Signal for Gain<T>
where
    T::Frame: Frame,
{
    type Frame = T::Frame;

    #[allow(clippy::float_cmp)]
    fn sample(&mut self, interval: f32, out: &mut [T::Frame]) {
        self.inner.sample(interval, out);
        let shared = f32::from_bits(self.shared.load(Ordering::Relaxed));
        if self.gain.target() != &shared {
            self.gain.set(shared);
        }
        if self.gain.progress() == 1.0 {
            let g = self.gain.get();
            if g != 1.0 {
                for x in out {
                    *x = scale(x, g);
                }
            }
            return;
        }
        for x in out {
            *x = scale(x, self.gain.get());
            self.gain.advance(interval / SMOOTHING_PERIOD);
        }
    }

    fn is_finished(&self) -> bool {
        self.inner.is_finished()
    }
}

#[inline]
fn scale<T: Frame>(x: &T, factor: f32) -> T {
    map(x, |x| x * factor)
}

#[inline]
fn map<T: Frame>(x: &T, mut f: impl FnMut(Sample) -> Sample) -> T {
    let mut out = T::ZERO;
    for (&x, o) in x.channels().iter().zip(out.channels_mut()) {
        *o = f(x);
    }
    out
}

/// Thread-safe control for a [`Gain`] filter
pub struct GainControl(Arc<AtomicU32>);

impl GainControl {
    /// Get the current amplification in decibels
    pub fn gain(&self) -> f32 {
        20.0 * self.amplitude_ratio().log10()
    }

    /// Amplify the signal by `db` decibels
    ///
    /// Perceptually linear. Negative values make the signal quieter.
    ///
    /// Equivalent to `self.set_amplitude_ratio(10.0f32.powf(db / 20.0))`.
    pub fn set_gain(&mut self, db: f32) {
        self.set_amplitude_ratio(10.0f32.powf(db / 20.0));
    }

    /// Get the current amplitude scaling factor
    pub fn amplitude_ratio(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }

    /// Scale the amplitude of the signal directly
    ///
    /// This is nonlinear in terms of both perception and power. Most users should prefer
    /// `set_gain`. Unlike `set_gain`, this method allows a signal to be completely zeroed out if
    /// needed, or even have its phase inverted with a negative factor.
    pub fn set_amplitude_ratio(&mut self, factor: f32) {
        self.0.store(factor.to_bits(), Ordering::Relaxed);
    }
}

/// Number of seconds over which to smooth a change in gain
const SMOOTHING_PERIOD: f32 = 0.1;
