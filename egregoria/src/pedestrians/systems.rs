use crate::engine_interaction::TimeInfo;
use crate::map_dynamic::Itinerary;
use crate::pedestrians::PedestrianComponent;
use crate::physics::{Collider, CollisionWorld, Kinematics, PhysicsObject, Transform};
use crate::rendering::meshrender_component::MeshRender;
use crate::utils::Restrict;
use geom::{angle_lerp, Vec2};
use map_model::{LaneKind, Map, PedestrianPath, Traversable, TraverseDirection, TraverseKind};
use specs::prelude::*;
use specs::shred::PanicHandler;
use std::borrow::Borrow;

#[derive(Default)]
pub struct PedestrianDecision;

#[derive(SystemData)]
pub struct PedestrianDecisionData<'a> {
    cow: Read<'a, CollisionWorld, PanicHandler>,
    map: Read<'a, Map, PanicHandler>,
    time: Read<'a, TimeInfo>,
    colliders: ReadStorage<'a, Collider>,
    itinerarys: WriteStorage<'a, Itinerary>,
    transforms: WriteStorage<'a, Transform>,
    kinematics: WriteStorage<'a, Kinematics>,
    pedestrians: WriteStorage<'a, PedestrianComponent>,
    mr: WriteStorage<'a, MeshRender>,
}

impl<'a> System<'a> for PedestrianDecision {
    type SystemData = PedestrianDecisionData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let cow: &CollisionWorld = data.cow.borrow();
        let map: &Map = data.map.borrow();
        let time: &TimeInfo = data.time.borrow();
        (
            &data.colliders,
            &mut data.itinerarys,
            &mut data.transforms,
            &mut data.kinematics,
            &mut data.pedestrians,
            &mut data.mr,
        )
            .join()
            .for_each(|(coll, it, trans, kin, pedestrian, mr)| {
                objective_update(it, trans, map, time);

                let (_, my_obj) = cow.get(coll.0).expect("Handle not in collision world");
                let neighbors = cow.query_around(trans.position(), 10.0);

                let objs = neighbors.map(|(id, pos)| {
                    (
                        Vec2::from(pos),
                        cow.get(id).expect("Handle not in collision world").1,
                    )
                });

                let (desired_v, desired_dir) =
                    calc_decision(pedestrian, trans, kin, map, my_obj, it, objs);

                walk_anim(pedestrian, mr, time, kin);
                physics(kin, trans, time, desired_v, desired_dir);
            });
    }
}

pub fn walk_anim(
    pedestrian: &mut PedestrianComponent,
    mr: &mut MeshRender,
    time: &TimeInfo,
    kin: &Kinematics,
) {
    let speed = kin.velocity.magnitude();
    pedestrian.walk_anim += 7.0 * speed * time.delta / pedestrian.walking_speed;

    let offset = pedestrian.walk_anim.cos()
        * 0.1
        * (speed * 2.0 - pedestrian.walking_speed).restrict(0.0, 1.0);

    mr.orders[0].as_rect_mut().offset.x = offset;
    mr.orders[1].as_rect_mut().offset.x = -offset;
}

const PEDESTRIAN_ACC: f32 = 1.0;

pub fn physics(
    kin: &mut Kinematics,
    trans: &mut Transform,
    time: &TimeInfo,
    desired_velocity: Vec2,
    desired_dir: Vec2,
) {
    let diff = desired_velocity - kin.velocity;
    let mag = diff.magnitude().min(time.delta * PEDESTRIAN_ACC);
    if mag > 0.0 {
        kin.velocity += diff.normalize_to(mag);
    }

    const ANG_VEL: f32 = 1.0;

    trans.set_direction(angle_lerp(
        trans.direction(),
        desired_dir,
        ANG_VEL * time.delta,
    ));
}

pub fn calc_decision<'a>(
    pedestrian: &mut PedestrianComponent,
    trans: &Transform,
    kin: &Kinematics,
    map: &Map,
    my_obj: &PhysicsObject,
    it: &Itinerary,
    neighs: impl Iterator<Item = (Vec2, &'a PhysicsObject)>,
) -> (Vec2, Vec2) {
    let objective = match it.get_point() {
        Some(x) => x,
        None => return (Vec2::ZERO, trans.direction()),
    };

    let position = trans.position();
    let direction = trans.direction();

    let delta_pos: Vec2 = objective - position;
    let dir_to_pos = match delta_pos.try_normalize() {
        Some(x) => x,
        None => return (Vec2::ZERO, trans.direction()),
    };

    let mut desired_v = dir_to_pos * pedestrian.walking_speed;

    for (his_pos, his_obj) in neighs {
        if his_pos == position {
            continue;
        }

        let towards_vec: Vec2 = his_pos - position;
        if let Some((towards_dir, dist)) = towards_vec.dir_dist() {
            let forward_boost = 1.0 + direction.dot(towards_dir).abs();

            desired_v += -towards_dir
                * 2.0
                * (-(dist - his_obj.radius - my_obj.radius) * 2.0).exp()
                * forward_boost;
        }
    }

    if !it.is_terminal() {
        if let Some(points) = it.get_travers().map(|x| x.raw_points(map)) {
            // Fixme: performance heavy on long curved roads which can have many points
            let projected = points.project(position);
            let lane_force = projected - trans.position();
            let m = lane_force.magnitude();
            desired_v += lane_force * m * 0.1;
        }
    }

    desired_v = desired_v.cap_magnitude(1.2 * pedestrian.walking_speed);

    let desired_dir = (dir_to_pos + kin.velocity).normalize();

    (desired_v, desired_dir)
}

pub fn objective_update(itinerary: &mut Itinerary, trans: &Transform, map: &Map, time: &TimeInfo) {
    if itinerary.has_ended(time.time) {
        let mut last_travers = itinerary.get_travers().copied();
        if last_travers.is_none() {
            last_travers = map
                .closest_lane(trans.position(), LaneKind::Walking)
                .map(|x| Traversable::new(TraverseKind::Lane(x), TraverseDirection::Forward));
        }

        *itinerary = next_objective(trans.position(), map, last_travers.as_ref())
            .unwrap_or_else(|| Itinerary::wait_until(time.time + 10.0));
    }
}

fn next_objective(pos: Vec2, map: &Map, last_travers: Option<&Traversable>) -> Option<Itinerary> {
    let l = map.get_random_lane(LaneKind::Walking, &mut rand::thread_rng())?;

    Itinerary::route(
        pos,
        *last_travers?,
        (
            l.id,
            l.points
                .point_along(rand::random::<f32>() * l.points.length()),
        ),
        map,
        &PedestrianPath,
    )
}
