use crate::map_dynamic::Itinerary;
use crate::pedestrians::Pedestrian;
use crate::physics::{Collider, CollisionWorld, Kinematics, PhysicsObject};
use crate::utils::time::GameTime;
use geom::{angle_lerp3, Transform, Vec2, Vec3};
use legion::system;
use map_model::{Map, TraverseDirection};

register_system!(pedestrian_decision);
#[system(par_for_each)]
pub fn pedestrian_decision(
    #[resource] cow: &CollisionWorld,
    #[resource] map: &Map,
    #[resource] time: &GameTime,
    coll: &Collider,
    it: &mut Itinerary,
    trans: &mut Transform,
    kin: &mut Kinematics,
    pedestrian: &mut Pedestrian,
) {
    let (_, my_obj) = cow.get(coll.0).expect("Handle not in collision world");
    let neighbors = cow.query_around(trans.position.xy(), 10.0);

    let objs =
        neighbors.map(|(id, pos)| (pos, cow.get(id).expect("Handle not in collision world").1));

    let (desired_v, desired_dir) = calc_decision(pedestrian, trans, kin, map, my_obj, it, objs);

    let speed = kin.velocity.magnitude();
    pedestrian.walk_anim += 7.0 * speed * time.delta / pedestrian.walking_speed;
    physics(kin, trans, time, desired_v, desired_dir);
}

const PEDESTRIAN_ACC: f32 = 1.5;

pub fn physics(
    kin: &mut Kinematics,
    trans: &mut Transform,
    time: &GameTime,
    desired_velocity: Vec3,
    desired_dir: Vec3,
) {
    let diff = desired_velocity - kin.velocity;
    let mag = diff.magnitude().min(time.delta * PEDESTRIAN_ACC);
    if mag > 0.0 {
        kin.velocity += diff.normalize_to(mag);
    }

    const ANG_VEL: f32 = 1.0;

    trans.dir = angle_lerp3(trans.dir, desired_dir, ANG_VEL * time.delta);
}

pub fn calc_decision<'a>(
    pedestrian: &mut Pedestrian,
    trans: &Transform,
    kin: &Kinematics,
    map: &Map,
    my_obj: &PhysicsObject,
    it: &Itinerary,
    neighs: impl Iterator<Item = (Vec2, &'a PhysicsObject)>,
) -> (Vec3, Vec3) {
    let objective = match it.get_point() {
        Some(x) => x,
        None => return (Vec3::ZERO, trans.dir),
    };

    let position = trans.position;
    let direction = trans.dir;

    let delta_pos: Vec3 = objective - position;
    let dir_to_pos = match delta_pos.try_normalize() {
        Some(x) => x,
        None => return (Vec3::ZERO, trans.dir),
    };

    let mut desired_v = dir_to_pos * pedestrian.walking_speed;

    for (his_pos, his_obj) in neighs {
        if his_pos == position.xy() {
            continue;
        }

        let towards_vec: Vec3 = his_pos.z(his_obj.height) - position;
        if let Some((towards_dir, dist)) = towards_vec.dir_dist() {
            let forward_boost = 1.0 + direction.dot(towards_dir).abs();

            desired_v += -towards_dir
                * 2.0
                * (-(dist - his_obj.radius - my_obj.radius) * 2.0).exp()
                * forward_boost;
        }
    }

    if !it.is_terminal() {
        if let Some((dir, points)) = it
            .get_travers()
            .and_then(|x| x.raw_points(map).map(|v| (x.dir, v)))
        {
            // Fixme: performance heavy on long curved roads which can have many points
            let (projected, _, proj_dir) = points.project_segment_dir(position);
            let walk_side = match dir {
                TraverseDirection::Forward => 1.0,
                TraverseDirection::Backward => -1.0,
            };

            let lane_force = (projected + proj_dir.perp_up() * walk_side) - trans.position;
            let m = lane_force.magnitude();

            desired_v += lane_force * m * 0.1;
        }
    }

    desired_v = desired_v.cap_magnitude(1.2 * pedestrian.walking_speed);

    let desired_dir = (dir_to_pos + kin.velocity).normalize();

    (desired_v, desired_dir)
}
