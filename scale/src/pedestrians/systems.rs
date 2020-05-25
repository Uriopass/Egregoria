use crate::engine_interaction::TimeInfo;
use crate::geometry::{Vec2, Vec2Impl};
use crate::map_model::{
    Itinerary, LaneKind, Map, PedestrianPath, Traversable, TraverseDirection, TraverseKind,
};
use crate::pedestrians::PedestrianComponent;
use crate::physics::{Collider, CollisionWorld, Kinematics, PhysicsObject, Transform};
use crate::rendering::meshrender_component::MeshRender;
use crate::utils::Restrict;
use cgmath::{Angle, InnerSpace, MetricSpace};
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
            &mut data.transforms,
            &mut data.kinematics,
            &mut data.pedestrians,
            &mut data.mr,
        )
            .join()
            .for_each(|(coll, trans, kin, pedestrian, mr)| {
                objective_update(pedestrian, trans, map, time.time);

                let my_obj = cow.get_obj(coll.0);
                let neighbors = cow.query_around(trans.position(), 10.0);

                let objs = neighbors.map(|obj| (obj.pos, cow.get_obj(obj.id)));

                let (desired_v, desired_dir) =
                    calc_decision(pedestrian, trans, kin, map, my_obj, objs);

                physics(pedestrian, kin, trans, mr, time, desired_v, desired_dir);
            });
    }
}

const PEDESTRIAN_ACC: f32 = 1.0;

pub fn physics(
    pedestrian: &mut PedestrianComponent,
    kin: &mut Kinematics,
    trans: &mut Transform,
    mr: &mut MeshRender,
    time: &TimeInfo,
    desired_velocity: Vec2,
    desired_dir: Vec2,
) {
    let diff = desired_velocity - kin.velocity;
    let mag = diff.magnitude().min(time.delta * PEDESTRIAN_ACC);
    if mag > 0.0 {
        let lol = diff.normalize_to(mag);
        kin.velocity += lol;
    }

    let speed = kin.velocity.magnitude();
    pedestrian.walk_anim += 7.0 * speed * time.delta / pedestrian.walking_speed;

    let offset = pedestrian.walk_anim.cos()
        * 0.1
        * (speed * 2.0 - pedestrian.walking_speed).restrict(0.0, 1.0);

    mr.orders[0].as_rect_mut().offset.x = offset;
    mr.orders[1].as_rect_mut().offset.x = -offset;

    let delta_ang = trans.direction().angle(desired_dir);
    let mut ang = vec2!(1.0, 0.0).angle(trans.direction());

    const ANG_VEL: f32 = 1.0;
    ang.0 += delta_ang
        .0
        .restrict(-ANG_VEL * time.delta, ANG_VEL * time.delta);

    trans.set_direction(vec2!(ang.cos(), ang.sin()));
}

pub fn calc_decision<'a>(
    pedestrian: &mut PedestrianComponent,
    trans: &Transform,
    kin: &Kinematics,
    map: &Map,
    my_obj: &PhysicsObject,
    neighs: impl Iterator<Item = (Vec2, &'a PhysicsObject)>,
) -> (Vec2, Vec2) {
    let objective = match pedestrian.itinerary.get_point() {
        Some(x) => x,
        None => return (vec2!(0.0, 0.0), trans.direction()),
    };

    let position = trans.position();
    let direction = trans.direction();

    let delta_pos: Vec2 = objective - position;
    let (dir_to_pos, _) = match delta_pos.dir_dist() {
        Some(x) => x,
        None => return (vec2!(0.0, 0.0), trans.direction()),
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

    if !pedestrian.itinerary.is_terminal() {
        if let Some(points) = pedestrian
            .itinerary
            .get_travers()
            .map(|x| x.raw_points(map))
        {
            if let Some(projected) = points.project(position) {
                let lane_force = projected - trans.position();
                let m = lane_force.magnitude();
                desired_v += lane_force * m * 0.1;
            }
        }
    }

    desired_v = desired_v.cap_magnitude(1.2 * pedestrian.walking_speed);

    let desired_dir = (dir_to_pos + kin.velocity).normalize();

    (desired_v, desired_dir)
}

pub fn objective_update(
    pedestrian: &mut PedestrianComponent,
    trans: &Transform,
    map: &Map,
    time: f64,
) {
    pedestrian.itinerary.check_validity(map);
    let mut last_travers = pedestrian.itinerary.get_travers().copied();

    if let Some(x) = pedestrian.itinerary.get_point() {
        if x.distance(trans.position()) > 3.0 {
            return;
        }
        pedestrian.itinerary.advance(map);
    }

    if pedestrian.itinerary.has_ended(time) {
        if last_travers.is_none() {
            last_travers = map
                .closest_lane(trans.position(), LaneKind::Walking)
                .map(|x| Traversable::new(TraverseKind::Lane(x), TraverseDirection::Forward));
        }

        let l = unwrap_or!(map.get_random_lane(LaneKind::Walking), return);

        pedestrian.itinerary = Itinerary::route(
            unwrap_or!(last_travers, return),
            (l.id, l.points.random_along().unwrap()),
            map,
            &PedestrianPath,
        );

        if pedestrian.itinerary.is_none() {
            println!("Pedestrian at {:?} couldn't find path", trans.position());
            pedestrian.itinerary = Itinerary::wait_until(time + 10.0);
        }
    }
}
