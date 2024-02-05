use crate::uiworld::UiWorld;
use engine::{AudioContext, AudioKind, Gain, GainControl};
use geom::{lerp, Camera, Vec2, AABB};
use oddio::{Cycle, Mixed};
use simulation::Simulation;

/// Ambient sounds
/// These are sounds that are played in the background
/// They are not tied to any entity
pub struct Ambient {
    wind: Option<(GainControl, Mixed)>,
    forest: Option<(GainControl, Mixed)>,
}

impl Ambient {
    pub fn new(ctx: &mut AudioContext) -> Self {
        let wind = ctx.play_with_control(
            "calm_wind",
            |s| {
                let (g_control, signal) = Gain::new(Cycle::new(s), 0.0);
                (g_control, signal)
            },
            AudioKind::Effect,
        );
        let forest = ctx.play_with_control(
            "forest",
            |s| {
                let (g_control, signal) = Gain::new(Cycle::new(s), 0.0);
                (g_control, signal)
            },
            AudioKind::Effect,
        );

        Self { wind, forest }
    }

    pub fn update(&mut self, sim: &Simulation, uiworld: &UiWorld) {
        let eye = uiworld.read::<Camera>().eye();
        let map = sim.map();

        let h = eye.z;

        // Wind
        let volume = lerp(0.1, 0.8, (h - 100.0) / 4000.0);
        if let Some(ref mut wind) = self.wind {
            wind.0.set_amplitude_ratio(volume);
        }

        // Forest
        let bbox = AABB::centered(eye.xy(), Vec2::splat(200.0));
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
            forest.0.set_amplitude_ratio(volume);
        }
    }
}
