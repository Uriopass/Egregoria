use crate::geometry::splines::Spline;
use crate::gui::InspectDragf;
use crate::interaction::Selectable;
use crate::map_interaction::Itinerary;
use crate::map_model::{
    LaneKind, Map, ParkingSpotID, Traversable, TraverseDirection, TraverseKind,
};
use crate::physics::{
    Collider, CollisionWorld, Kinematics, PhysicsGroup, PhysicsObject, Transform,
};
use crate::rendering::assets::{AssetID, AssetRender};
use crate::rendering::Color;
use crate::utils::rand_world;
use crate::RandProvider;
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};
use specs::{Builder, Entity, World, WorldExt};
use specs::{Component, DenseVecStorage};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum VehicleState {
    Parked,
    ParkedToRoad(Spline, f32),
    Driving,
    RoadToPark(Spline, f32),
}

debug_inspect_impl!(VehicleState);

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum VehicleKind {
    Car,
    Bus,
}

#[derive(Component, Debug, Inspect, Serialize, Deserialize)]
pub struct VehicleComponent {
    #[inspect(proxy_type = "InspectDragf")]
    pub ang_velocity: f32,
    #[inspect(proxy_type = "InspectDragf")]
    pub wait_time: f32,

    #[inspect(skip)]
    pub park_spot: Option<ParkingSpotID>,

    pub state: VehicleState,
    pub kind: VehicleKind,
}

impl Default for VehicleComponent {
    fn default() -> Self {
        Self {
            wait_time: 0.0,
            ang_velocity: 0.0,
            kind: VehicleKind::Car,
            state: VehicleState::Driving,
            park_spot: None,
        }
    }
}

impl VehicleKind {
    pub fn width(self) -> f32 {
        match self {
            VehicleKind::Car => 4.5,
            VehicleKind::Bus => 9.0,
        }
    }

    pub fn height(self) -> f32 {
        match self {
            VehicleKind::Car => 2.0,
            VehicleKind::Bus => 2.0,
        }
    }

    pub fn acceleration(self) -> f32 {
        match self {
            VehicleKind::Car => 3.0,
            VehicleKind::Bus => 2.0,
        }
    }

    pub fn deceleration(self) -> f32 {
        match self {
            VehicleKind::Car => 9.0,
            VehicleKind::Bus => 9.0,
        }
    }

    pub fn min_turning_radius(self) -> f32 {
        match self {
            VehicleKind::Car => 3.0,
            VehicleKind::Bus => 5.0,
        }
    }

    pub fn cruising_speed(self) -> f32 {
        match self {
            VehicleKind::Car => 15.0,
            VehicleKind::Bus => 10.0,
        }
    }

    pub fn ang_acc(self) -> f32 {
        match self {
            VehicleKind::Car => 1.0,
            VehicleKind::Bus => 0.8,
        }
    }
}

pub fn spawn_new_vehicle(world: &mut World) {
    let r: f32 = rand_world(world);

    let map = world.read_resource::<Map>();

    let lane = unwrap_or!(
        map.get_random_lane(
            LaneKind::Driving,
            &mut world.write_resource::<RandProvider>().rng,
        ),
        return
    );

    let (pos, dir) = lane.points.point_dir_along(r * lane.points.length());

    let (segment, _) = lane.points.project_segment(pos);

    let pos = Transform::new_cos_sin(pos, dir);

    let mut it = Itinerary::simple(
        Traversable::new(TraverseKind::Lane(lane.id), TraverseDirection::Forward),
        &map,
    );
    for _ in 0..segment {
        it.advance(&map);
    }

    drop(map);
    make_vehicle_entity(world, pos, VehicleComponent::new(VehicleKind::Car), it);
}

pub fn make_vehicle_entity(
    world: &mut World,
    trans: Transform,
    vehicle: VehicleComponent,
    it: Itinerary,
) -> Entity {
    let coworld = world.get_mut::<CollisionWorld>().unwrap();
    let h = coworld.insert(
        trans.position(),
        PhysicsObject {
            dir: trans.direction(),
            speed: 0.0,
            radius: vehicle.kind.width() / 2.0,
            group: PhysicsGroup::Vehicles,
        },
    );

    world
        .create_entity()
        .with(AssetRender {
            id: AssetID::CAR,
            hide: false,
            scale: 4.5,
            tint: get_random_car_color(),
            z: 0.7,
        })
        .with(trans)
        .with(Kinematics::from_mass(1000.0))
        .with(Collider(h))
        .with(Selectable::default())
        .with(vehicle)
        .with(it)
        .build()
}

pub fn delete_vehicle_entity(world: &mut World, e: Entity) {
    {
        let handle = world.read_component::<Collider>().get(e).unwrap().0;
        let mut coworld = world.write_resource::<CollisionWorld>();
        coworld.remove(handle);
    }
    world.delete_entity(e).unwrap();
}

pub fn get_random_car_color() -> Color {
    let car_colors: [(Color, f32); 9] = [
        (Color::from_hex(0x22_22_22), 0.22),  // Black
        (Color::from_hex(0xff_ff_ff), 0.19),  // White
        (Color::from_hex(0x66_66_66), 0.17),  // Gray
        (Color::from_hex(0xb8_b8_b8), 0.14),  // Silver
        (Color::from_hex(0x1a_3c_70), 0.1),   // Blue
        (Color::from_hex(0xd8_22_00), 0.1),   // Red
        (Color::from_hex(0x7c_4b_24), 0.02),  // Brown
        (Color::from_hex(0xd4_c6_78), 0.015), // Gold
        (Color::from_hex(0x72_cb_19), 0.015), // Green
    ];

    let total: f32 = car_colors.iter().map(|x| x.1).sum();

    let r = rand::random::<f32>() * total;
    let mut partial = 0.0;
    for (col, freq) in &car_colors {
        partial += freq;
        if partial >= r {
            return *col;
        }
    }
    unreachable!();
}

impl VehicleComponent {
    pub fn new(kind: VehicleKind) -> VehicleComponent {
        Self {
            kind,
            ..Default::default()
        }
    }
}

enum_inspect_impl!(VehicleKind; VehicleKind::Car, VehicleKind::Bus);
