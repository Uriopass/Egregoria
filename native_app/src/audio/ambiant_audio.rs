use crate::audio::{AudioContext, AudioHandle};
use egregoria::rendering::immediate::ImmediateDraw;
use egregoria::Egregoria;
use geom::{lerp, vec2, Camera};
use map_model::Map;

pub struct AmbientAudio {
    wind: AudioHandle,
    forest: AudioHandle,
}

impl AmbientAudio {
    pub fn new(ctx: &mut AudioContext) -> Self {
        let wind = ctx.play_with_control("calm_wind", true);
        ctx.set_volume(wind, 0.0);

        let forest = ctx.play_with_control("forest", true);
        ctx.set_volume(forest, 0.0);

        Self { wind, forest }
    }

    pub fn update(&self, goria: &mut Egregoria, ctx: &mut AudioContext, delta: f32) {
        let delta = delta.min(0.1);
        let camera = goria.read::<Camera>();
        let map = goria.read::<Map>();

        let h = camera.position.z;

        // Wind
        let volume = lerp(0.01, 0.2, (h - 100.0) / 10000.0);
        ctx.set_volume_smooth(self.wind, volume, delta * 0.05);

        // Forest
        let bbox = camera.get_screen_box();

        let mut volume = lerp(0.2, 0.0, h / 2000.0);
        if h < 2000.0
            && map
                .trees
                .grid
                .query_aabb(bbox.ll(), bbox.ur())
                .next()
                .is_none()
        {
            volume = 0.0;
        }
        ctx.set_volume_smooth(self.forest, volume, delta * 0.05);
    }
}
