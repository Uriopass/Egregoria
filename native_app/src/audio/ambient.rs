use crate::uiworld::UiWorld;
use common::AudioKind;
use engine::{AudioContext, ControlHandle, Stereo};
use geom::{lerp, Camera, Vec2, AABB};
use oddio::{Cycle, Gain};
use simulation::Simulation;

/// Ambient sounds
/// These are sounds that are played in the background
/// They are not tied to any entity
pub struct Ambient {
    wind: Option<ControlHandle<Gain<Cycle<Stereo>>>>,
    forest: Option<ControlHandle<Gain<Cycle<Stereo>>>>,
}

impl Ambient {
    pub fn new(ctx: &mut AudioContext) -> Self {
        let wind = ctx.play_with_control(
            "calm_wind",
            |s| {
                let mut g = Gain::new(Cycle::new(s));
                g.set_amplitude_ratio(0.0);
                g
            },
            AudioKind::Effect,
        );
        let forest = ctx.play_with_control(
            "forest",
            |s| {
                let mut g = Gain::new(Cycle::new(s));
                g.set_amplitude_ratio(0.0);
                g
            },
            AudioKind::Effect,
        );

        Self { wind, forest }
    }

    pub fn update(&mut self, sim: &Simulation, uiworld: &mut UiWorld) {
        let eye = uiworld.read::<Camera>().eye();
        let map = sim.map();

        let h = eye.z;

        // Wind
        let volume = lerp(0.1, 0.8, (h - 100.0) / 4000.0);
        if let Some(ref mut wind) = self.wind {
            wind.control::<Gain<_>, _>().set_amplitude_ratio(volume);
        }

        // Forest
        let bbox = AABB::new(eye.xy() - Vec2::splat(100.0), eye.xy() + Vec2::splat(100.0));
        let mut volume = lerp(1.0, 0.0, h / 300.0);

        if volume > 0.0 {
            let mut matches = 0;

            for _ in map.environment.trees.query(bbox.ll, bbox.ur) {
                matches += 1;
                if matches > 50 {
                    break;
                }
            }
            volume *= matches as f32 / 50.0;
        }
        if let Some(ref mut forest) = self.forest {
            forest.control::<Gain<_>, _>().set_amplitude_ratio(volume);
        }
    }
}
