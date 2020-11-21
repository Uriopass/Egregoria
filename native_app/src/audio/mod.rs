pub mod ambiant_audio;

use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use slotmap::{new_key_type, DenseSlotMap};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

new_key_type! {
    pub struct AudioHandle;
}

// We allow dead_code because we need to keep OutputStream alive for it to work
#[allow(dead_code)]
pub struct AudioContext {
    out: Option<OutputStream>,
    out_handle: Option<OutputStreamHandle>,
    sinks: DenseSlotMap<AudioHandle, Sink>,
    dummy: AudioHandle,
    cache: HashMap<&'static str, &'static [u8]>,
}

impl AudioContext {
    pub fn new() -> Self {
        let mut sinks = DenseSlotMap::with_key();
        let dummy = sinks.insert(Sink::new_idle().0);

        let (out, out_handle) = match rodio::OutputStream::try_default() {
            Ok(x) => x,
            Err(e) => {
                log::error!("Couldn't initialize audio because of {}", e);
                return Self {
                    out: None,
                    out_handle: None,
                    sinks,
                    dummy,
                    cache: Default::default(),
                };
            }
        };

        Self {
            out: Some(out),
            out_handle: Some(out_handle),
            sinks,
            dummy,
            cache: Default::default(),
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

    pub fn play(&mut self, name: &'static str) {
        if let Some(ref h) = self.out_handle {
            if let Some(x) = Self::get(&mut self.cache, name) {
                let dec = rodio::Decoder::new(std::io::Cursor::new(x)).unwrap();
                let _ = h.play_raw(dec.convert_samples());
            }
        }
    }

    pub fn play_with_control(&mut self, name: &'static str, repeat: bool) -> AudioHandle {
        if let Some(ref h) = self.out_handle {
            if let Some(x) = Self::get(&mut self.cache, name) {
                let dec = rodio::Decoder::new(std::io::Cursor::new(x)).unwrap();
                let sink = rodio::Sink::try_new(h).unwrap();
                if repeat {
                    sink.append(dec.repeat_infinite());
                } else {
                    sink.append(dec);
                }
                return self.sinks.insert(sink);
            }
        }
        self.dummy
    }

    pub fn set_volume(&self, handle: AudioHandle, volume: f32) {
        if let Some(x) = self.sinks.get(handle) {
            let volume = volume.max(0.0).min(2.0);
            x.set_volume(volume);
        }
    }

    pub fn set_volume_smooth(&self, handle: AudioHandle, volume: f32, max_change: f32) {
        if let Some(x) = self.sinks.get(handle) {
            let cur_volume = x.volume();
            let volume = volume.max(0.0).min(2.0);
            self.set_volume(
                handle,
                cur_volume + (volume - cur_volume).max(-max_change).min(max_change),
            )
        }
    }
}
