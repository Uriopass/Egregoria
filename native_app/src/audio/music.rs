use crate::audio::{AudioContext, AudioHandle};
use common::AudioKind;
use rodio::Source;
use std::time::{Duration, Instant};

const TRACKS: &[&str] = &["music2", "music1"];

pub struct Music {
    track_id: usize,
    time_between_tracks: Duration,
    last_played: Instant,
    cur_track: Option<AudioHandle>,
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
        if let Some(x) = self.cur_track {
            if !ctx.is_over(x) {
                return;
            }
            self.cur_track = None;
        }

        if self.last_played.elapsed() > self.time_between_tracks {
            self.track_id = (self.track_id + 1) % TRACKS.len();
            self.cur_track = Some(ctx.play_with_control(
                TRACKS[self.track_id],
                |s| s.fade_in(Duration::new(5, 0)).amplify(0.5),
                AudioKind::Music,
                false,
            ));
            log::info!("playing soundtrack {}", TRACKS[self.track_id]);
            self.last_played = Instant::now();
        }
    }
}
