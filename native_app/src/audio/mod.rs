pub mod ambient;
pub mod music;

mod car_sounds;
mod unique_sink;

use crate::audio::ambient::Ambient;
use crate::audio::music::Music;
use crate::audio::unique_sink::UniqueSink;
use crate::gui::Settings;
use common::AudioKind;
use egregoria::Egregoria;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sample, Source};
use slotmap::{new_key_type, DenseSlotMap};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read};
use std::sync::atomic::{AtomicU32, Ordering};

pub struct GameAudio {
    music: Music,
    ambiant: Ambient,
}

impl GameAudio {
    pub fn new(ctx: &mut AudioContext) -> Self {
        Self {
            music: Music::new(),
            ambiant: Ambient::new(ctx),
        }
    }

    pub fn update(&mut self, goria: &mut Egregoria, ctx: &mut AudioContext, delta: f32) {
        self.music.update(ctx);
        self.ambiant.update(goria, ctx, delta);
    }
}

new_key_type! {
    pub struct AudioHandle;
}

pub struct PlayingSink {
    sink: UniqueSink,
    kind: AudioKind,
    volume: AtomicU32,
}

impl PlayingSink {
    pub fn set_volume(&self, ctx: &AudioContext, volume: f32) {
        self.volume
            .store(volume.to_bits(), std::sync::atomic::Ordering::SeqCst);
        self.sink.set_volume(ctx.g_volume(self.kind) * volume);
    }
}

// We allow dead_code because we need to keep OutputStream alive for it to work
#[allow(dead_code)]
pub struct AudioContext {
    out: Option<OutputStream>,
    out_handle: Option<OutputStreamHandle>,
    sinks: DenseSlotMap<AudioHandle, PlayingSink>,
    dummy: AudioHandle,
    cache: HashMap<&'static str, &'static [u8]>,

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

    pub fn g_volume(&self, kind: AudioKind) -> f32 {
        match kind {
            AudioKind::Music => self.music_volume,
            AudioKind::Effect => self.effect_volume,
            AudioKind::Ui => self.ui_volume,
        }
    }

    fn get(
        cache: &mut HashMap<&'static str, &'static [u8]>,
        name: &'static str,
    ) -> Option<&'static [u8]> {
        let e = cache.entry(name);

        match e {
            Entry::Occupied(x) => Some(x.get()),
            Entry::Vacant(v) => {
                let mut f = match File::open(format!("assets/sounds/{}.ogg", name)) {
                    Ok(x) => x,
                    Err(e) => {
                        log::error!("Could not load sound {}: {}", name, e);
                        return None;
                    }
                };

                let mut buf = vec![];
                let _ = f.read_to_end(&mut buf);
                Some(v.insert(buf.leak()))
            }
        }
    }

    pub fn play(&mut self, name: &'static str, kind: AudioKind) {
        if let Some(ref h) = self.out_handle {
            if let Some(x) = Self::get(&mut self.cache, name) {
                let dec = rodio::Decoder::new(std::io::Cursor::new(x)).unwrap();
                let _ = h.play_raw(dec.convert_samples().amplify(self.g_volume(kind)));
            }
        }
    }

    pub fn play_with_control<S>(
        &mut self,
        name: &'static str,
        transform: impl FnOnce(Decoder<Cursor<&'static [u8]>>) -> S,
        kind: AudioKind,
    ) -> AudioHandle
    where
        S: Source + Send + 'static,
        S::Item: Sample + Send,
    {
        if let Some(ref h) = self.out_handle {
            if let Some(x) = Self::get(&mut self.cache, name) {
                let dec = rodio::Decoder::new(std::io::Cursor::new(x)).unwrap();
                let sink = UniqueSink::try_new(h, transform(dec)).unwrap();

                sink.set_volume(self.g_volume(kind));
                return self.sinks.insert(PlayingSink {
                    sink,
                    kind,
                    volume: 1.0f32.to_bits().into(),
                });
            }
        }
        self.dummy
    }

    pub fn set_volume(&self, handle: AudioHandle, volume: f32) {
        if let Some(x) = self.sinks.get(handle) {
            let volume = volume.max(0.0).min(2.0);
            x.set_volume(self, volume);
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

    pub fn set_settings(&mut self, settings: Settings) {
        let mut changed = false;

        let ui_volume = settings.ui_volume_percent / 100.0;
        if (self.ui_volume - ui_volume).abs() > f32::EPSILON {
            self.ui_volume = ui_volume;
            changed = true;
        }

        let music_volume = settings.music_volume_percent / 100.0;
        if (self.music_volume - music_volume).abs() > f32::EPSILON {
            self.music_volume = music_volume;
            changed = true;
        }

        let effect_volume = settings.effects_volume_percent / 100.0;
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
