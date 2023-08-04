use crate::audio::{AudioContext, AudioKind, ControlHandle, Stereo};
use crate::uiworld::UiWorld;
use egregoria::map::Terrain;
use egregoria::Egregoria;
use geom::{lerp, vec2, Camera, Vec2, AABB};
use oddio::{Cycle, Gain};

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

    pub fn update(&mut self, goria: &Egregoria, uiworld: &mut UiWorld) {
        let eye = uiworld.read::<Camera>().eye();
        let map = goria.map();

        let h = eye.z;

        // Wind
        let volume = lerp(0.1, 0.8, (h - 100.0) / 4000.0);
        if let Some(ref mut wind) = self.wind {
            wind.control::<Gain<_>, _>().set_amplitude_ratio(volume);
        }

        // Forest
        let bbox = AABB::new(eye.xy() - Vec2::splat(100.0), eye.xy() + Vec2::splat(100.0));
        let mut volume = lerp(1.0, 0.0, h / 600.0);

        let ll = bbox.ll;
        let ur = bbox.ur;
        let ul = vec2(ll.x, ur.y);
        let lr = vec2(ur.x, ll.y);
        let tree_check = [
            ll.lerp(ur, 0.25),
            ll.lerp(ur, 0.75),
            ul.lerp(lr, 0.25),
            ul.lerp(lr, 0.75),
        ];

        if volume > 0.0 {
            let matches = tree_check
                .iter()
                .filter(|&&p| {
                    map.terrain
                        .chunks
                        .get(&Terrain::cell(p))
                        .map(|x| x.trees.len() > 10)
                        .unwrap_or_default()
                })
                .count();

            volume *= matches as f32 / 4.0;
        }
        if let Some(ref mut forest) = self.forest {
            forest.control::<Gain<_>, _>().set_amplitude_ratio(volume);
        }
    }
}
