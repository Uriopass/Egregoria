use crate::geometry::Vec2;
use crate::gui::{InspectDragf, InspectVec2};
use crate::interaction::Selectable;
use crate::map_model::{Itinerary, LaneKind, Map};
use crate::physics::{
    Collider, CollisionWorld, Kinematics, PhysicsGroup, PhysicsObject, Transform,
};
use crate::rendering::assets::{AssetID, AssetRender};
use crate::rendering::meshrender_component::{MeshRender, RectRender};
use crate::rendering::Color;
use crate::utils::rand_det;
use cgmath::InnerSpace;
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};
use specs::{Builder, Entity, World, WorldExt};
use specs::{Component, DenseVecStorage};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum VehicleKind {
    Car,
    Bus,
}

#[derive(Component, Debug, Inspect, Serialize, Deserialize)]
pub struct VehicleComponent {
    pub itinerary: Itinerary,
    #[inspect(proxy_type = "InspectDragf")]
    pub desired_speed: f32,
    #[inspect(proxy_type = "InspectVec2")]
    pub desired_dir: Vec2,
    #[inspect(proxy_type = "InspectDragf")]
    pub ang_velocity: f32,
    #[inspect(proxy_type = "InspectDragf")]
    pub wait_time: f32,

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

    pub fn build_mr(self, mr: &mut MeshRender) {
        let width = self.width();
        let height = self.height();

        match self {
            VehicleKind::Car => {
                mr.add(RectRender {
                    width,
                    height,
                    color: get_random_car_color(),
                    ..Default::default()
                })
                .add(RectRender {
                    width: 0.4,
                    height: 1.8,
                    offset: [-1.7, 0.0].into(),
                    color: Color::BLACK,
                })
                .add(RectRender {
                    width: 1.0,
                    height: 1.6,
                    offset: [0.8, 0.0].into(),
                    color: Color::BLACK,
                })
                .add(RectRender {
                    width: 2.7,
                    height: 0.15,
                    offset: [-0.4, 0.85].into(),
                    color: Color::BLACK,
                })
                .add(RectRender {
                    width: 2.7,
                    height: 0.15,
                    offset: [-0.4, -0.85].into(),
                    color: Color::BLACK,
                })
                .add(RectRender {
                    width: 0.4,
                    height: 0.15,
                    offset: [2.1, -0.7].into(),
                    color: Color::BLACK,
                })
                .add(RectRender {
                    width: 0.4,
                    height: 0.15,
                    offset: [2.1, 0.7].into(),
                    color: Color::BLACK,
                });
            }
            VehicleKind::Bus => {
                mr.add(RectRender {
                    width: self.width(),
                    height: self.height(),
                    color: Color::ORANGE,
                    ..Default::default()
                });
            }
        }
    }
}

pub fn spawn_new_vehicle(world: &mut World) {
    let map = world.read_resource::<Map>();

    if let Some(lane) = map.get_random_lane(LaneKind::Driving) {
        if let [a, b, ..] = lane.points.as_slice() {
            let diff = b - a;

            let mut pos = Transform::new(*a + rand_det::<f32>() * diff);
            pos.set_direction(diff.normalize());

            let it = Itinerary::none();

            drop(map);
            make_vehicle_entity(world, pos, VehicleComponent::new(it, VehicleKind::Car));
        }
    }
}

pub fn make_vehicle_entity(
    world: &mut World,
    trans: Transform,
    vehicle: VehicleComponent,
) -> Entity {
    let mut mr = MeshRender::empty(0.3);
    vehicle.kind.build_mr(&mut mr);

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
        //.with(mr)
        .with(AssetRender {
            id: AssetID::CAR,
            hide: false,
            scale: 4.5,
            tint: get_random_car_color(),
        })
        .with(trans)
        .with(Kinematics::from_mass(1000.0))
        .with(vehicle)
        .with(Collider(h))
        .with(Selectable::default())
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

    let r = rand_det::<f32>() * total;
    let mut partial = 0.0;
    for (col, freq) in &car_colors {
        partial += freq;
        if partial >= r {
            return *col;
        }
    }
    unreachable!();
}

impl Default for VehicleComponent {
    fn default() -> Self {
        Self {
            itinerary: Default::default(),
            desired_speed: 0.0,
            desired_dir: vec2!(1.0, 0.0),
            wait_time: 0.0,
            ang_velocity: 0.0,
            kind: VehicleKind::Car,
        }
    }
}

impl VehicleComponent {
    pub fn new(itinerary: Itinerary, kind: VehicleKind) -> VehicleComponent {
        Self {
            itinerary,
            kind,
            ..Default::default()
        }
    }
}

enum_inspect_impl!(VehicleKind; VehicleKind::Car, VehicleKind::Bus);
