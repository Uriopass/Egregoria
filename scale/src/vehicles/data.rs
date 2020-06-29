use crate::engine_interaction::TimeInfo;
use crate::geometry::splines::Spline;
use crate::gui::InspectDragf;
use crate::interaction::Selectable;
use crate::map_interaction::{Itinerary, ParkingManagement};
use crate::map_model::{LaneKind, Map, ParkingSpotID};
use crate::physics::{Collider, CollisionWorld, Kinematics, Transform};
use crate::rendering::assets::{AssetID, AssetRender};
use crate::rendering::Color;
use crate::utils::rand_world;
use crate::RandProvider;
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};
use specs::{Builder, Entity, World, WorldExt};
use specs::{Component, DenseVecStorage};

pub const TIME_TO_PARK: f32 = 5.0;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum VehicleState {
    Parked(ParkingSpotID),
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
    let r: f64 = rand_world(world);

    let map = world.read_resource::<Map>();

    let time = world.read_resource::<TimeInfo>().time;
    let it = Itinerary::wait_until(time + r * 5.0);

    let pm = world.read_resource::<ParkingManagement>();

    let rl = unwrap_or!(
        map.get_random_lane(
            LaneKind::Parking,
            &mut world.write_resource::<RandProvider>().rng
        ),
        return
    );
    let spot_id = unwrap_or!(
        pm.reserve_near(rl.id, rl.points.random_along().0, &map),
        return
    );

    let spot = map.parking.get(spot_id).unwrap(); // Unwrap ok: Gotten using reserve_near
    let pos = Transform::new_cos_sin(spot.pos, spot.orientation);
    drop(map);
    drop(pm);

    make_vehicle_entity(
        world,
        pos,
        VehicleComponent::new(VehicleKind::Car, spot_id),
        it,
    );
}

pub fn make_vehicle_entity(
    world: &mut World,
    trans: Transform,
    vehicle: VehicleComponent,
    it: Itinerary,
) -> Entity {
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
        .with(Selectable::default())
        .with(vehicle)
        .with(it)
        .build()
}

pub fn delete_vehicle_entity(world: &mut World, e: Entity) {
    if let Some(&Collider(handle)) = world.read_component::<Collider>().get(e) {
        let mut coworld = world.write_resource::<CollisionWorld>();
        coworld.remove(handle);
    }
    world.delete_entity(e).unwrap(); // Unwrap ok: only point where car can be deleted
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
    pub fn new(kind: VehicleKind, spot: ParkingSpotID) -> VehicleComponent {
        Self {
            ang_velocity: 0.0,
            wait_time: 0.0,
            park_spot: Some(spot),
            state: VehicleState::Parked(spot),
            kind,
        }
    }
}

enum_inspect_impl!(VehicleKind; VehicleKind::Car, VehicleKind::Bus);
