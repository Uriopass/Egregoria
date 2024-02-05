use crate::uiworld::UiWorld;
use engine::{AudioContext, AudioKind, Gain, GainControl};
use flat_spatial::grid::GridHandle;
use geom::{Camera, Vec2, AABB};
use oddio::{Cycle, Mixed, Seek, Speed, SpeedControl};
use simulation::transportation::TransportGrid;
use simulation::Simulation;
use slotmapd::SecondaryMap;

/// CarSound is the sound of a single car
pub struct CarSound {
    road: Option<(SpeedControl, GainControl, Mixed)>,
    engine: Option<(SpeedControl, GainControl, Mixed)>,
}

/// CarSounds are sounds that are played when cars are near the player
/// They are tied to a car entity
pub struct CarSounds {
    sounds: SecondaryMap<GridHandle, CarSound>,
    generic_car_sound: Option<GainControl>,
}

impl CarSounds {
    pub fn new(ctx: &mut AudioContext) -> Self {
        Self {
            sounds: SecondaryMap::new(),
            generic_car_sound: ctx
                .play_with_control(
                    "car_loop",
                    |x| Gain::new(Cycle::new(x), 0.0),
                    AudioKind::Effect,
                )
                .map(|x| x.0),
        }
    }

    pub fn update(&mut self, sim: &Simulation, uiworld: &UiWorld, ctx: &mut AudioContext) {
        let transport_grid = sim.read::<TransportGrid>();
        let campos = uiworld.read::<Camera>().eye();
        let cambox = AABB::centered(campos.xy(), Vec2::splat(200.0));

        const HEAR_RADIUS: f32 = 200.0;

        #[cfg(not(debug_assertions))]
        const MAX_SOUNDS: usize = 30;
        #[cfg(debug_assertions)]
        const MAX_SOUNDS: usize = 1;

        let mut to_remove = vec![];

        for (h, _) in &self.sounds {
            if let Some((pos, _)) = transport_grid.get(h) {
                if pos.z0().is_close(campos, HEAR_RADIUS) {
                    continue;
                }
            }

            to_remove.push(h);
        }

        for h in to_remove {
            let cs = self.sounds.remove(h).unwrap();
            if let Some((_, _, mut mixed)) = cs.road {
                mixed.stop();
            }
            if let Some((_, _, mut mixed)) = cs.engine {
                mixed.stop();
            }
        }

        // Gather
        for (h, _) in transport_grid.query_around(
            campos.xy(),
            (HEAR_RADIUS * HEAR_RADIUS - campos.z * campos.z)
                .max(0.0)
                .sqrt(),
        ) {
            let (pos, obj) = transport_grid.get(h).unwrap();
            if !matches!(
                obj.group,
                simulation::transportation::TransportationGroup::Vehicles
            ) {
                continue;
            }

            if self.sounds.len() >= MAX_SOUNDS {
                break;
            }

            if !self.sounds.contains_key(h) {
                let engine = ctx
                    .play_with_control(
                        "car_engine",
                        |x| {
                            let mut cycle = Cycle::new(x);
                            cycle.seek(common::rand::rand2(pos.x, pos.y));
                            let (g_control, signal) = Gain::new(cycle, 0.0);
                            let (speed_control, signal) = Speed::new(signal);
                            ((speed_control, g_control), signal)
                        },
                        AudioKind::Effect,
                    )
                    .map(|((a, b), c)| (a, b, c));

                let road = ctx
                    .play_with_control(
                        "car_loop",
                        |x| {
                            let mut cycle = Cycle::new(x);
                            cycle.seek(common::rand::rand2(pos.x, pos.y));
                            let (g_control, signal) = Gain::new(cycle, 0.0);
                            let (speed_control, signal) = Speed::new(signal);
                            ((speed_control, g_control), signal)
                        },
                        AudioKind::Effect,
                    )
                    .map(|((a, b), c)| (a, b, c));

                self.sounds.insert(h, CarSound { road, engine });
            }
        }

        // Update
        for (h, cs) in &mut self.sounds {
            let (pos, obj) = transport_grid.get(h).unwrap(); // Unwrap ok: checked it existed before

            let his_speed = (obj.speed * obj.dir).z0();
            let dir_to_me = (campos - pos.z(campos.z * 0.5)).normalize();

            let speed_to_me = his_speed.dot(dir_to_me);
            let boost = 300.0 / (300.0 - speed_to_me);

            if let Some((ref mut speed, ref mut gain, _)) = cs.road {
                gain.set_amplitude_ratio(obj.speed.sqrt() * 3.0 / pos.z0().distance(campos));
                speed.set_speed(boost)
            }

            if let Some((ref mut speed, ref mut gain, _)) = cs.engine {
                gain.set_amplitude_ratio(obj.speed.sqrt() / pos.z0().distance(campos));
                speed.set_speed(boost)
            }
        }

        if campos.z < 1000.0 {
            let cars_on_screen = transport_grid
                .query_aabb(cambox.ll, cambox.ur)
                .filter_map(|(h, _)| transport_grid.get(h))
                .filter(|(_, obj)| {
                    matches!(
                        obj.group,
                        simulation::transportation::TransportationGroup::Vehicles
                    )
                })
                .count();
            if let Some(ref mut s) = self.generic_car_sound {
                s.set_amplitude_ratio(
                    ((cars_on_screen as f32).min(100.0) / 100.0 * (1.0 - campos.z / 1000.0))
                        .min(0.03),
                );
            }
        } else if let Some(ref mut s) = self.generic_car_sound {
            s.set_amplitude_ratio(0.0);
        }
    }
}
