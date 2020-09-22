use crate::engine_interaction::{Selectable, TimeInfo};
use crate::map_dynamic::{Itinerary, ParkingManagement};
use crate::physics::{Collider, CollisionWorld, Kinematics, PhysicsGroup, PhysicsObject};
use crate::rendering::assets::{AssetID, AssetRender};
use crate::rendering::Color;
use crate::utils::rand_world;
use crate::{Egregoria, RandProvider};
use geom::{Spline, Transform};
use imgui_inspect::InspectDragf;
use imgui_inspect_derive::*;
use legion::Entity;
use map_model::{LaneKind, Map, ParkingSpotID};
use serde::{Deserialize, Serialize};

/// The duration for the parking animation.
pub const TIME_TO_PARK: f32 = 4.0;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct VehicleID(pub Entity);

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum VehicleState {
    Parked(ParkingSpotID),
    Driving,
    RoadToPark(Spline, f32, ParkingSpotID),
}

debug_inspect_impl!(VehicleState);

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum VehicleKind {
    Car,
    Bus,
}

#[derive(Clone, Debug, Inspect, Serialize, Deserialize)]
pub struct Vehicle {
    #[inspect(proxy_type = "InspectDragf")]
    pub ang_velocity: f32,
    #[inspect(proxy_type = "InspectDragf")]
    pub wait_time: f32,

    pub state: VehicleState,
    pub kind: VehicleKind,
}

pub fn put_vehicle_in_coworld(goria: &mut Egregoria, w: f32, trans: Transform) -> Collider {
    Collider(goria.write::<CollisionWorld>().insert(
        trans.position(),
        PhysicsObject {
            dir: trans.direction(),
            speed: 0.0,
            radius: w * 0.5,
            group: PhysicsGroup::Vehicles,
        },
    ))
}

impl VehicleKind {
    pub fn width(self) -> f32 {
        match self {
            VehicleKind::Car => 4.5,
            VehicleKind::Bus => 9.0,
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

pub fn spawn_parked_vehicle(goria: &mut Egregoria) {
    let r: f64 = rand_world(goria);

    let map = goria.read::<Map>();

    let time = goria.read::<TimeInfo>().time;
    let it = Itinerary::wait_until(time + r * 5.0);

    let pm = goria.read::<ParkingManagement>();

    let rl = unwrap_or!(
        map.random_lane(LaneKind::Parking, &mut *goria.write::<RandProvider>()),
        return
    );
    let spot_id = unwrap_or!(
        pm.reserve_near(
            rl.id,
            rl.points
                .point_along(rand::random::<f32>() * rl.points.length()),
            &map
        ),
        return
    );

    let pos = map.parking.get(spot_id).unwrap().trans; // Unwrap ok: Gotten using reserve_near

    drop(map);
    drop(pm);

    make_vehicle_entity(
        goria,
        pos,
        Vehicle::new(VehicleKind::Car, spot_id),
        it,
        false,
    );
}

pub fn make_vehicle_entity(
    goria: &mut Egregoria,
    trans: Transform,
    vehicle: Vehicle,
    it: Itinerary,
    mk_collider: bool,
) -> Entity {
    let w = vehicle.kind.width();
    let e = goria.world.push((
        AssetRender {
            id: AssetID::CAR,
            hide: false,
            scale: w,
            tint: get_random_car_color(),
            z: 0.7,
        },
        trans,
        Kinematics::from_mass(1000.0),
        Selectable::default(),
        vehicle,
        it,
    ));

    if mk_collider {
        let c = put_vehicle_in_coworld(goria, w, trans);
        goria.world.entry(e).unwrap().add_component(c);
    }

    e
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

impl Vehicle {
    pub fn new(kind: VehicleKind, spot: ParkingSpotID) -> Vehicle {
        Self {
            ang_velocity: 0.0,
            wait_time: 0.0,
            state: VehicleState::Parked(spot),
            kind,
        }
    }
}

enum_inspect_impl!(VehicleKind; VehicleKind::Car, VehicleKind::Bus);
