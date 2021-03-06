mod ambient;
mod car_sounds;
mod music;
mod unique_sink;

use crate::audio::ambient::Ambient;
use crate::audio::car_sounds::CarSounds;
use crate::audio::music::Music;
use crate::audio::unique_sink::UniqueSink;
use crate::gui::windows::settings::Settings;
use crate::uiworld::UiWorld;
use common::AudioKind;
use common::FastMap;
use egregoria::Egregoria;
use rodio::source::Buffered;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sample, Source};
use slotmap::{new_key_type, DenseSlotMap};
use std::collections::hash_map::Entry;
use std::io::Cursor;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

pub struct GameAudio {
    music: Music,
    ambiant: Ambient,
    carsounds: CarSounds,
}

impl GameAudio {
    pub fn new(ctx: &mut AudioContext) -> Self {
        Self {
            music: Music::new(),
            ambiant: Ambient::new(ctx),
            carsounds: CarSounds::new(ctx),
        }
    }

    pub fn update(
        &mut self,
        goria: &Egregoria,
        uiworld: &mut UiWorld,
        ctx: &mut AudioContext,
        delta: f32,
    ) {
        self.music.update(ctx);
        self.ambiant.update(goria, uiworld, ctx, delta);
        self.carsounds.update(goria, uiworld, ctx, delta);
    }
}

new_key_type! {
    pub struct AudioHandle;
}

pub struct PlayingSink {
    sink: UniqueSink,
    kind: AudioKind,
    volume: AtomicU32,
    complex: bool,
}

impl PlayingSink {
    pub fn set_volume(&self, ctx: &AudioContext, volume: f32) {
        self.volume
            .store(volume.to_bits(), std::sync::atomic::Ordering::SeqCst);
        self.sink.set_volume(ctx.g_volume(self.kind) * volume);
    }
}

type StoredAudio = Buffered<Decoder<Cursor<&'static [u8]>>>;

// We allow dead_code because we need to keep OutputStream alive for it to work
#[allow(dead_code)]
pub struct AudioContext {
    out: Option<OutputStream>,
    out_handle: Option<OutputStreamHandle>,
    sinks: DenseSlotMap<AudioHandle, PlayingSink>,
    dummy: AudioHandle,
    cache: FastMap<&'static str, StoredAudio>,

    music_volume: f32,
    effect_volume: f32,
    ui_volume: f32,
}

impl AudioContext {
    pub fn new() -> Self {
        let (out, out_handle) = match rodio::OutputStream::try_default() {
            Ok(x) => x,
            Err(e) => {
                log::error!("Couldn't initialize audio because of {}", e);
                return Self {
                    out: None,
                    out_handle: None,
                    sinks: DenseSlotMap::with_key(),
                    dummy: AudioHandle::default(),
                    cache: Default::default(),

                    music_volume: 1.0,
                    effect_volume: 1.0,
                    ui_volume: 1.0,
                };
            }
        };

        Self {
            out: Some(out),
            out_handle: Some(out_handle),
            sinks: DenseSlotMap::with_key(),
            dummy: AudioHandle::default(),
            cache: Default::default(),
            music_volume: 1.0,
            effect_volume: 1.0,
            ui_volume: 1.0,
        }
    }

    pub fn update(&mut self) {
        let to_kill: Vec<_> = self
            .sinks
            .iter()
            .filter(|(_, sink)| sink.sink.is_dead())
            .map(|(id, _)| id)
            .collect();
        for v in to_kill {
            self.sinks.remove(v);
        }
    }

    pub fn g_volume(&self, kind: AudioKind) -> f32 {
        match kind {
            AudioKind::Music => self.music_volume,
            AudioKind::Effect => self.effect_volume,
            AudioKind::Ui => self.ui_volume,
        }
    }

    fn get(
        cache: &mut FastMap<&'static str, StoredAudio>,
        name: &'static str,
    ) -> Option<StoredAudio> {
        let e = cache.entry(name);

        match e {
            Entry::Occupied(x) => Some(x.get().clone()),
            Entry::Vacant(v) => {
                let buf = match std::fs::read(format!("assets/sounds/{}.ogg", name)) {
                    Ok(x) => x,
                    Err(e) => {
                        log::error!("Could not load sound {}: {}", name, e);
                        return None;
                    }
                };

                let s = rodio::Decoder::new(std::io::Cursor::new(&*buf.leak()))
                    .unwrap()
                    .buffered();
                Some(v.insert(s).clone())
            }
        }
    }

    pub fn play(&mut self, name: &'static str, kind: AudioKind) {
        if let Some(ref h) = self.out_handle {
            if let Some(x) = Self::get(&mut self.cache, name) {
                log::info!("playing {}", name);
                let _ = h.play_raw(x.convert_samples().amplify(self.g_volume(kind)));
            }
        }
    }

    pub fn play_with_control<S>(
        &mut self,
        name: &'static str,
        transform: impl FnOnce(StoredAudio) -> S,
        kind: AudioKind,
        complex: bool,
    ) -> AudioHandle
    where
        S: Source + Send + 'static,
        S::Item: Sample + Send,
    {
        if let Some(ref h) = self.out_handle {
            if let Some(x) = Self::get(&mut self.cache, name) {
                let sink = UniqueSink::try_new(h, transform(x), complex).unwrap();
                return self.sinks.insert(PlayingSink {
                    sink,
                    kind,
                    volume: 0.0f32.to_bits().into(),
                    complex,
                });
            }
        }
        self.dummy
    }

    pub fn stop(&self, handle: AudioHandle) {
        if let Some(x) = self.sinks.get(handle) {
            x.sink.stop();
        }
    }

    pub fn set_volume(&self, handle: AudioHandle, volume: f32) {
        if let Some(x) = self.sinks.get(handle) {
            let volume = volume.max(0.0).min(2.0);
            x.set_volume(self, volume);
        }
    }

    pub fn set_speed(&self, handle: AudioHandle, speed: f32) {
        if let Some(x) = self.sinks.get(handle) {
            if !x.complex {
                log::warn!("trying to set speed of {:?} but it is not a complex sound. This won't do anything", handle);
                return;
            }
            let speed = speed.max(0.0).min(2.0);
            x.sink.set_speed(speed);
        }
    }

    pub fn is_over(&self, handle: AudioHandle) -> bool {
        if let Some(x) = self.sinks.get(handle) {
            x.sink.is_dead()
        } else {
            true
        }
    }

    pub fn set_volume_smooth(&self, handle: AudioHandle, volume: f32, max_change: f32) {
        if let Some(x) = self.sinks.get(handle) {
            let cur_volume = f32::from_bits(x.volume.load(Ordering::SeqCst));
            let volume = volume.max(0.0).min(2.0);
            self.set_volume(
                handle,
                cur_volume + (volume - cur_volume).max(-max_change).min(max_change),
            )
        }
    }

    pub fn set_settings(&mut self, settings: &Settings) {
        let mut changed = false;

        let ui_volume = (settings.ui_volume_percent / 100.0).powi(2);
        if (self.ui_volume - ui_volume).abs() > f32::EPSILON {
            self.ui_volume = ui_volume;
            changed = true;
        }

        let music_volume = (settings.music_volume_percent / 100.0).powi(2);
        if (self.music_volume - music_volume).abs() > f32::EPSILON {
            self.music_volume = music_volume;
            changed = true;
        }

        let effect_volume = (settings.effects_volume_percent / 100.0).powi(2);
        if (self.effect_volume - effect_volume).abs() > f32::EPSILON {
            self.effect_volume = effect_volume;
            changed = true;
        }

        if !changed {
            return;
        }

        for sink in self.sinks.values() {
            sink.set_volume(self, f32::from_bits(sink.volume.load(Ordering::SeqCst)));
        }
    }
}

struct PrintOnFirstSample<S: Source<Item = f32>> {
    s: S,
    printed: bool,
}

trait SourceExt: Source<Item = f32> + Sized {
    fn print_on_first(self) -> PrintOnFirstSample<Self>;
}

impl<S: Source<Item = f32>> SourceExt for S {
    fn print_on_first(self) -> PrintOnFirstSample<S> {
        PrintOnFirstSample {
            s: self,
            printed: false,
        }
    }
}

impl<S: Source<Item = f32>> Iterator for PrintOnFirstSample<S> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.printed {
            self.printed = true;
            log::info!("first sample");
        }
        self.s.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.s.size_hint()
    }
}

impl<S: Source<Item = f32>> Source for PrintOnFirstSample<S> {
    fn current_frame_len(&self) -> Option<usize> {
        self.s.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.s.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.s.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.s.total_duration()
    }
}
