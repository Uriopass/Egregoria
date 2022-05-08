use crate::engine_interaction::Selectable;
use crate::map_dynamic::{Itinerary, ParkingManagement, SpotReservation};
use crate::physics::{Collider, CollisionWorld, Kinematics, PhysicsGroup, PhysicsObject};
use crate::utils::par_command_buffer::ComponentDrop;
use crate::utils::rand_provider::RandProvider;
use crate::utils::time::GameInstant;
use crate::Egregoria;
use geom::Transform;
use geom::{Color, Spline3, Vec3};
use hecs::Entity;
use imgui_inspect::InspectDragf;
use imgui_inspect_derive::Inspect;
use resources::Resources;
use serde::{Deserialize, Serialize};

/// The duration for the parking animation.
pub const TIME_TO_PARK: f32 = 4.0;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct VehicleID(pub Entity);

debug_inspect_impl!(VehicleID);

#[derive(Debug, Serialize, Deserialize)]
pub enum VehicleState {
    Parked(SpotReservation),
    Driving,
    /// Panicked when it notices it's in a gridlock
    Panicking(GameInstant),
    RoadToPark(Spline3, f32, SpotReservation),
}

debug_inspect_impl!(VehicleState);

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum VehicleKind {
    Car,
    Truck,
    Bus,
}

#[derive(Debug, Serialize, Deserialize, Inspect)]
pub struct Vehicle {
    #[inspect(proxy_type = "InspectDragf")]
    pub ang_velocity: f32,
    #[inspect(proxy_type = "InspectDragf")]
    pub wait_time: f32,

    pub state: VehicleState,
    pub kind: VehicleKind,
    pub tint: Color,

    /// Used to detect gridlock
    pub flag: u64,
}

impl ComponentDrop for Vehicle {
    fn drop(&mut self, res: &mut Resources, _: Entity) {
        if let VehicleState::Parked(resa) | VehicleState::RoadToPark(_, _, resa) =
            std::mem::replace(&mut self.state, VehicleState::Driving)
        {
            res.get_mut::<ParkingManagement>().unwrap().free(resa);
        }
    }
}

#[must_use]
pub fn put_vehicle_in_coworld(goria: &mut Egregoria, w: f32, trans: Transform) -> Collider {
    Collider(goria.write::<CollisionWorld>().insert(
        trans.position.xy(),
        PhysicsObject {
            dir: trans.dir.xy(),
            radius: w * 0.5,
            group: PhysicsGroup::Vehicles,
            ..Default::default()
        },
    ))
}

impl VehicleKind {
    pub fn width(self) -> f32 {
        match self {
            VehicleKind::Car => 4.5,
            VehicleKind::Truck => 6.0,
            VehicleKind::Bus => 9.0,
        }
    }

    pub fn acceleration(self) -> f32 {
        match self {
            VehicleKind::Car => 3.0,
            VehicleKind::Truck => 2.5,
            VehicleKind::Bus => 2.0,
        }
    }

    pub fn deceleration(self) -> f32 {
        match self {
            VehicleKind::Car | VehicleKind::Bus | VehicleKind::Truck => 6.0,
        }
    }

    pub fn min_turning_radius(self) -> f32 {
        match self {
            VehicleKind::Car => 1.5,
            VehicleKind::Truck => 3.0,
            VehicleKind::Bus => 4.0,
        }
    }

    pub fn speed_factor(self) -> f32 {
        match self {
            VehicleKind::Car => 1.0,
            VehicleKind::Truck | VehicleKind::Bus => 0.8,
        }
    }

    pub fn ang_acc(self) -> f32 {
        match self {
            VehicleKind::Car => 1.0,
            VehicleKind::Truck => 0.9,
            VehicleKind::Bus => 0.8,
        }
    }
}

pub fn unpark(goria: &mut Egregoria, vehicle: VehicleID) {
    let mut v = unwrap_ret!(goria.comp_mut::<Vehicle>(vehicle.0));
    let w = v.kind.width();

    if let VehicleState::Parked(spot) = std::mem::replace(&mut (*v).state, VehicleState::Driving) {
        drop(v);
        goria.write::<ParkingManagement>().free(spot);
    } else {
        drop(v);
        log::warn!("Trying to unpark {:?} that wasn't parked", vehicle);
    }

    let trans = *unwrap_ret!(goria.comp::<Transform>(vehicle.0));
    let coll = put_vehicle_in_coworld(goria, w, trans);
    goria.add_comp(vehicle.0, coll);
}

pub fn spawn_parked_vehicle(
    goria: &mut Egregoria,
    kind: VehicleKind,
    near: Vec3,
) -> Option<VehicleID> {
    let map = goria.map();

    let it = Itinerary::none();

    let mut pm = goria.write::<ParkingManagement>();

    let spot_id = pm.reserve_near(near, &map)?;

    let pos = spot_id.get(&map.parking).unwrap().trans; // Unwrap ok: Gotten using reserve_near

    drop(map);
    drop(pm);

    let tint = match kind {
        VehicleKind::Car => get_random_car_color(&mut *goria.write::<RandProvider>()),
        _ => Color::WHITE,
    };

    Some(VehicleID(make_vehicle_entity(
        goria,
        pos,
        Vehicle::new(kind, spot_id, tint),
        it,
        false,
    )))
}

pub fn make_vehicle_entity(
    goria: &mut Egregoria,
    trans: Transform,
    vehicle: Vehicle,
    it: Itinerary,
    mk_collider: bool,
) -> Entity {
    let w = vehicle.kind.width();
    let e = goria.world.spawn((
        trans,
        Kinematics::default(),
        Selectable::default(),
        vehicle,
        it,
    ));

    if mk_collider {
        let c = put_vehicle_in_coworld(goria, w, trans);
        #[allow(clippy::unwrap_used)] // literally just added to the world
        let _ = goria.world.insert_one(e, c);
    }

    e
}

pub fn get_random_car_color(r: &mut RandProvider) -> Color {
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

    let r = r.random::<f32>() * total;
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
    pub fn new(kind: VehicleKind, spot: SpotReservation, tint: Color) -> Vehicle {
        Self {
            ang_velocity: 0.0,
            wait_time: 0.0,
            state: VehicleState::Parked(spot),
            kind,
            tint,
            flag: 0,
        }
    }
}

debug_inspect_impl!(VehicleKind);
