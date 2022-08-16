use crate::audio::{AudioContext, BaseSignal, ControlHandle, FadeIn};
use common::AudioKind;
use oddio::{FramesSignal, Stop};
use std::time::{Duration, Instant};

const TRACKS: &[&str] = &["music2", "music1"];

pub struct Music {
    track_id: usize,
    time_between_tracks: Duration,
    last_played: Instant,
    cur_track: Option<ControlHandle<FadeIn<BaseSignal>>>,
}

impl Music {
    pub fn new() -> Self {
        Self {
            track_id: 0,
            time_between_tracks: Duration::new(5, 0),
            last_played: Instant::now(),
            cur_track: None,
        }
    }

    pub fn update(&mut self, ctx: &mut AudioContext) {
        if !ctx.is_all_ready() {
            return;
        }
        if let Some(ref mut x) = self.cur_track {
            if !x.control::<Stop<_>, _>().is_stopped() {
                return;
            }
            self.cur_track = None;
        }

        if self.last_played.elapsed() > self.time_between_tracks {
            self.track_id = (self.track_id + 1) % TRACKS.len();
            let h = ctx.play_with_control(
                TRACKS[self.track_id],
                |s| FadeIn::new(FramesSignal::new(s, 0.0), 5.0),
                AudioKind::Music,
            );
            self.cur_track = h;
            log::info!("playing soundtrack {}", TRACKS[self.track_id]);
            self.last_played = Instant::now();
        }
    }
}
