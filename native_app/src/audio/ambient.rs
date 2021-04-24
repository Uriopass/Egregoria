use crate::audio::{AudioContext, AudioHandle, AudioKind};
use crate::uiworld::UiWorld;
use egregoria::Egregoria;
use geom::{lerp, vec2, Camera, Vec2, AABB};
use rodio::Source;

pub struct Ambient {
    wind: AudioHandle,
    forest: AudioHandle,
}

impl Ambient {
    pub fn new(ctx: &mut AudioContext) -> Self {
        let wind = ctx.play_with_control(
            "calm_wind",
            |s| s.repeat_infinite(),
            AudioKind::Effect,
            false,
        );
        ctx.set_volume(wind, 0.0);

        let forest =
            ctx.play_with_control("forest", |s| s.repeat_infinite(), AudioKind::Effect, false);
        ctx.set_volume(forest, 0.0);

        Self { wind, forest }
    }

    pub fn update(
        &self,
        goria: &Egregoria,
        uiworld: &mut UiWorld,
        ctx: &mut AudioContext,
        delta: f32,
    ) {
        let delta = delta.min(0.1);
        let eye = uiworld.read::<Camera>().eye();
        let map = goria.map();

        let h = eye.z;

        // Wind
        let volume = lerp(0.1, 0.8, (h - 100.0) / 4000.0);
        ctx.set_volume_smooth(self.wind, volume, delta * 0.05);

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
                .filter_map(|&p| map.trees.grid.query_around(p, h * 0.2).next())
                .count();

            volume *= matches as f32 / 4.0;
        }
        ctx.set_volume_smooth(self.forest, volume, delta * 0.2);
    }
}
