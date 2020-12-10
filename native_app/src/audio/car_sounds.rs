use crate::audio::{AudioContext, AudioHandle};
use common::AudioKind;
use egregoria::physics::CollisionWorld;
use egregoria::Egregoria;
use flat_spatial::grid::GridHandle;
use geom::Camera;
use rodio::Source;
use slotmap::SecondaryMap;
use std::time::Duration;

pub struct CarSound {
    road: AudioHandle,
    engine: AudioHandle,
}

pub struct CarSounds {
    sounds: SecondaryMap<GridHandle, CarSound>,
    generic_car_sound: AudioHandle,
}

impl CarSounds {
    pub fn new(ctx: &mut AudioContext) -> Self {
        Self {
            sounds: SecondaryMap::new(),
            generic_car_sound: ctx.play_with_control(
                "car_loop",
                |x| x.repeat_infinite(),
                AudioKind::Effect,
                false,
            ),
        }
    }

    pub fn update(&mut self, goria: &Egregoria, ctx: &mut AudioContext, delta: f32) {
        let coworld = goria.read::<CollisionWorld>();
        let cam = goria.read::<Camera>();
        let campos = cam.position;
        let cambox = cam.get_screen_box().expand(100.0);

        const HEAR_RADIUS: f32 = 200.0;

        #[cfg(not(debug_assertions))]
        const MAX_SOUNDS: usize = 30;
        #[cfg(debug_assertions)]
        const MAX_SOUNDS: usize = 10;

        let mut to_remove = vec![];

        for (h, _) in &self.sounds {
            if let Some((pos, _)) = coworld.get(h) {
                if pos.z(0.0).is_close(campos, HEAR_RADIUS) {
                    continue;
                }
            }

            to_remove.push(h);
        }

        for h in to_remove {
            let cs = self.sounds.remove(h).unwrap();
            ctx.stop(cs.road);
            ctx.stop(cs.engine);
        }

        // Gather
        for (h, _) in coworld.query_around(
            campos.xy(),
            (HEAR_RADIUS * HEAR_RADIUS - campos.z * campos.z)
                .max(0.0)
                .sqrt(),
        ) {
            let (pos, obj) = coworld.get(h).unwrap();
            if !matches!(obj.group, egregoria::physics::PhysicsGroup::Vehicles) {
                continue;
            }

            if self.sounds.len() >= MAX_SOUNDS {
                break;
            }

            if !self.sounds.contains_key(h) {
                let engine = ctx.play_with_control(
                    "car_engine",
                    |x| {
                        x.repeat_infinite().skip_duration(Duration::from_millis(
                            (common::rand::rand2(pos.x, pos.y) * 1000.0) as u64,
                        ))
                    },
                    AudioKind::Effect,
                    true,
                );

                let road = ctx.play_with_control(
                    "car_loop",
                    |x| {
                        x.repeat_infinite().skip_duration(Duration::from_millis(
                            (common::rand::rand2(pos.x, pos.y) * 1000.0) as u64,
                        ))
                    },
                    AudioKind::Effect,
                    true,
                );

                self.sounds.insert(h, CarSound { road, engine });
            }
        }

        // Update
        for (h, cs) in &mut self.sounds {
            let (pos, obj) = coworld.get(h).unwrap(); // Unwrap ok: checked it existed before

            let his_speed = (obj.speed * obj.dir).z(0.0);
            let dir_to_me = (campos - pos.z(campos.z * 0.5)).normalize();

            let speed_to_me = his_speed.dot(dir_to_me);
            let boost = 300.0 / (300.0 - speed_to_me);

            ctx.set_volume(
                cs.road,
                obj.speed.sqrt() * 3.0 / pos.z(0.0).distance(campos),
            );
            ctx.set_speed(cs.road, boost);

            ctx.set_volume(cs.engine, obj.speed.sqrt() / pos.z(0.0).distance(campos));
            ctx.set_speed(cs.engine, boost);
        }

        if campos.z < 1000.0 {
            let cars_on_screen = coworld
                .query_aabb(cambox.ll, cambox.ur)
                .filter_map(|(h, _)| coworld.get(h))
                .filter(|(_, obj)| matches!(obj.group, egregoria::physics::PhysicsGroup::Vehicles))
                .count();
            ctx.set_volume_smooth(
                self.generic_car_sound,
                ((cars_on_screen as f32).min(100.0) / 100.0 * (1.0 - campos.z / 1000.0)).min(0.03),
                delta,
            )
        } else {
            ctx.set_volume_smooth(self.generic_car_sound, 0.0, delta);
        }
    }
}
