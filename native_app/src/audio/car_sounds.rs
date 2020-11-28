use crate::audio::{AudioContext, AudioHandle};
use common::AudioKind;
use rodio::Source;

pub struct CarSounds {
    pub speed: f32,
    pub sound: AudioHandle,
}

impl CarSounds {
    pub fn new(ctx: &mut AudioContext) -> Self {
        Self {
            speed: 0.0,
            sound: ctx.play_with_control(
                "car_loop",
                |x| x.repeat_infinite(),
                AudioKind::Effect,
                true,
            ),
        }
    }

    pub fn update(&mut self, ctx: &mut AudioContext) {
        //ctx.set_volume(self.sound, 1.0);
        //ctx.set_speed(self.sound, ctx.ui_volume * 2.0);
        //        let speed = inline_tweak::tweak!(1.0);
    }
}
