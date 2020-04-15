use crate::map_model::{Map, Traversable, TraverseDirection, TraverseKind};
use crate::pedestrians::PedestrianComponent;
use crate::physics::{CollisionWorld, Kinematics, PhysicsObject, Transform};
use crate::utils::{Choose, Restrict};
use cgmath::{vec2, InnerSpace, MetricSpace, Vector2, Zero};
use specs::prelude::*;
use specs::shred::PanicHandler;
use std::borrow::Borrow;

#[derive(Default)]
pub struct PedestrianDecision;

#[derive(SystemData)]
pub struct PedestrianDecisionData<'a> {
    cow: Read<'a, CollisionWorld, PanicHandler>,
    map: Read<'a, Map, PanicHandler>,
    transforms: WriteStorage<'a, Transform>,
    kinematics: WriteStorage<'a, Kinematics>,
    pedestrians: WriteStorage<'a, PedestrianComponent>,
}

impl<'a> System<'a> for PedestrianDecision {
    type SystemData = PedestrianDecisionData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let cow: &CollisionWorld = data.cow.borrow();
        let map: &Map = data.map.borrow();
        (
            &mut data.transforms,
            &mut data.kinematics,
            &mut data.pedestrians,
        )
            .join()
            .for_each(|(trans, kin, pedestrian)| {
                objective_update(pedestrian, trans, map);

                let neighbors = cow.query_around(trans.position(), 10.0);

                let objs = neighbors.map(|obj| (obj.pos, cow.get_obj(obj.id)));

                calc_decision(pedestrian, trans, kin, objs);
            });
    }
}

pub fn objective_update(pedestrian: &mut PedestrianComponent, trans: &Transform, map: &Map) {
    if let Some(x) = pedestrian.itinerary.get_point() {
        if x.distance(trans.position()) > 2.0 {
            return;
        }
        pedestrian.itinerary.advance(map);
    }

    if pedestrian.itinerary.has_ended() {
        let t = *unwrap_ret!(pedestrian.itinerary.get_travers());
        match t.kind {
            TraverseKind::Lane(l) => {
                let arrived = &map.intersections()[map.lanes()[l].dst];

                let neighs = arrived.turns_adirectional(l);

                /*println!("--- {:?}", l);
                for x in neighs.iter() {
                    println!("{:?}", x);
                }*/

                let turn = unwrap_ret!(neighs.choose());

                //println!("Choose {:?}", turn);
                let direction = if turn.id.src == l {
                    TraverseDirection::Forward
                } else {
                    TraverseDirection::Backward
                };

                pedestrian.itinerary.set_simple(
                    Traversable::new(TraverseKind::Turn(turn.id), direction),
                    map,
                );
            }
            TraverseKind::Turn(turn) => {
                let arrived_at = &map.lanes()[match t.dir {
                    TraverseDirection::Forward => turn.dst,
                    TraverseDirection::Backward => turn.src,
                }];

                let dir_if_take_lane = if arrived_at.src == turn.parent {
                    TraverseDirection::Forward
                } else {
                    TraverseDirection::Backward
                };

                let inter = &map.intersections()[turn.parent];

                let mut traversables = vec![Traversable::new(
                    TraverseKind::Lane(arrived_at.id),
                    dir_if_take_lane,
                )];

                for turn_inter in inter.turns_adirectional(arrived_at.id) {
                    if turn_inter.id == turn {
                        continue;
                    }
                    let direction = if turn_inter.id.src == arrived_at.id {
                        TraverseDirection::Forward
                    } else {
                        TraverseDirection::Backward
                    };

                    traversables.push(Traversable::new(
                        TraverseKind::Turn(turn_inter.id),
                        direction,
                    ));
                }

                pedestrian
                    .itinerary
                    .set_simple(*traversables.choose().unwrap(), map);
            }
        }
    }
}

pub fn calc_decision<'a>(
    pedestrian: &mut PedestrianComponent,
    trans: &mut Transform,
    kin: &mut Kinematics,
    neighs: impl Iterator<Item = (Vector2<f32>, &'a PhysicsObject)>,
) {
    let objective = unwrap_ret!(pedestrian.itinerary.get_point());
    let position = trans.position();
    let direction = trans.direction();

    let delta_pos: Vector2<f32> = objective - position;
    let dist_to_pos = delta_pos.magnitude();
    let dir_to_pos: Vector2<f32> = delta_pos / dist_to_pos;

    let mut v: Vector2<f32> = dir_to_pos * pedestrian.walking_speed;

    for (his_pos, his_obj) in neighs {
        if his_pos == position {
            continue;
        }

        let towards_vec = his_pos - position;
        let dist = towards_vec.magnitude();
        let towards_dir: Vector2<f32> = towards_vec / dist;

        let forward_boost = 1.0 + direction.dot(towards_dir).abs();

        v += -towards_dir * (-(dist - his_obj.radius).max(0.0) / 1.5).exp() * forward_boost;
    }

    v += 0.1 * vec2(rand::random::<f32>(), rand::random());

    let s = v.magnitude().restrict(0.0, 1.3 * pedestrian.walking_speed);

    kin.velocity = v.normalize_to(s);
    if !kin.velocity.is_zero() {
        trans.set_direction(kin.velocity.normalize());
    }
}
