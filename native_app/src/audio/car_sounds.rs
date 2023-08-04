use crate::audio::{AudioContext, ControlHandle, Stereo};
use crate::uiworld::UiWorld;
use common::AudioKind;
use egregoria::physics::CollisionWorld;
use egregoria::Egregoria;
use flat_spatial::grid::GridHandle;
use geom::{Camera, AABB};
use oddio::{Cycle, Gain, Seek, Speed, Stop};
use slotmapd::SecondaryMap;

/// CarSound is the sound of a single car
pub struct CarSound {
    road: Option<ControlHandle<Speed<Gain<Cycle<Stereo>>>>>,
    engine: Option<ControlHandle<Speed<Gain<Cycle<Stereo>>>>>,
}

/// CarSounds are sounds that are played when cars are near the player
/// They are tied to a car entity
pub struct CarSounds {
    sounds: SecondaryMap<GridHandle, CarSound>,
    generic_car_sound: Option<ControlHandle<Gain<Cycle<Stereo>>>>,
}

impl CarSounds {
    pub fn new(ctx: &mut AudioContext) -> Self {
        Self {
            sounds: SecondaryMap::new(),
            generic_car_sound: ctx.play_with_control(
                "car_loop",
                |x| {
                    let mut g = Gain::new(Cycle::new(x));
                    g.set_amplitude_ratio(0.0);
                    g
                },
                AudioKind::Effect,
            ),
        }
    }

    pub fn update(&mut self, goria: &Egregoria, uiworld: &mut UiWorld, ctx: &mut AudioContext) {
        let coworld = goria.read::<CollisionWorld>();
        let campos = uiworld.read::<Camera>().eye();
        let cambox = AABB::new(campos.xy(), campos.xy()).expand(100.0);

        const HEAR_RADIUS: f32 = 200.0;

        #[cfg(not(debug_assertions))]
        const MAX_SOUNDS: usize = 30;
        #[cfg(debug_assertions)]
        const MAX_SOUNDS: usize = 1;

        let mut to_remove = vec![];

        for (h, _) in &self.sounds {
            if let Some((pos, _)) = coworld.get(h) {
                if pos.z0().is_close(campos, HEAR_RADIUS) {
                    continue;
                }
            }

            to_remove.push(h);
        }

        for h in to_remove {
            let cs = self.sounds.remove(h).unwrap();
            if let Some(mut road) = cs.road {
                road.control::<Stop<_>, _>().stop();
            }
            if let Some(mut engine) = cs.engine {
                engine.control::<Stop<_>, _>().stop();
            }
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
                        let cycle = Cycle::new(x);
                        cycle.seek(common::rand::rand2(pos.x, pos.y));
                        let mut g = Gain::new(cycle);
                        g.set_amplitude_ratio(0.0);
                        Speed::new(g)
                    },
                    AudioKind::Effect,
                );

                let road = ctx.play_with_control(
                    "car_loop",
                    |x| {
                        let cycle = Cycle::new(x);
                        cycle.seek(common::rand::rand2(pos.x, pos.y));
                        let mut g = Gain::new(cycle);
                        g.set_amplitude_ratio(0.0);
                        Speed::new(g)
                    },
                    AudioKind::Effect,
                );

                self.sounds.insert(h, CarSound { road, engine });
            }
        }

        // Update
        for (h, cs) in &mut self.sounds {
            let (pos, obj) = coworld.get(h).unwrap(); // Unwrap ok: checked it existed before

            let his_speed = (obj.speed * obj.dir).z0();
            let dir_to_me = (campos - pos.z(campos.z * 0.5)).normalize();

            let speed_to_me = his_speed.dot(dir_to_me);
            let boost = 300.0 / (300.0 - speed_to_me);

            if let Some(ref mut road) = cs.road {
                road.control::<Gain<_>, _>()
                    .set_amplitude_ratio(obj.speed.sqrt() * 3.0 / pos.z0().distance(campos));
                road.control::<Speed<_>, _>().set_speed(boost)
            }

            if let Some(ref mut engine) = cs.road {
                engine
                    .control::<Gain<_>, _>()
                    .set_amplitude_ratio(obj.speed.sqrt() / pos.z0().distance(campos));
                engine.control::<Speed<_>, _>().set_speed(boost)
            }
        }

        if campos.z < 1000.0 {
            let cars_on_screen = coworld
                .query_aabb(cambox.ll, cambox.ur)
                .filter_map(|(h, _)| coworld.get(h))
                .filter(|(_, obj)| matches!(obj.group, egregoria::physics::PhysicsGroup::Vehicles))
                .count();
            if let Some(ref mut s) = self.generic_car_sound {
                s.control::<Gain<_>, _>().set_amplitude_ratio(
                    ((cars_on_screen as f32).min(100.0) / 100.0 * (1.0 - campos.z / 1000.0))
                        .min(0.03),
                );
            }
        } else if let Some(ref mut s) = self.generic_car_sound {
            s.control::<Gain<_>, _>().set_amplitude_ratio(0.0);
        }
    }
}
