use crate::geometry::Vec2;
use crate::gui::{InspectDragf, InspectVec2Rotation};
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use imgui_inspect_derive::*;
use specs::{Component, VecStorage, World, WorldExt};

pub mod systems;

mod kinematics;
mod transform;

use flat_spatial::densegrid::DenseGridHandle;
pub use kinematics::*;
pub use transform::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PhysicsGroup {
    Unknown,
    Vehicles,
    Pedestrians,
}

enum_inspect_impl!(PhysicsGroup; PhysicsGroup::Unknown, PhysicsGroup::Vehicles, PhysicsGroup::Pedestrians);

#[derive(Clone, Copy, Inspect)]
pub struct PhysicsObject {
    #[inspect(proxy_type = "InspectVec2Rotation")]
    pub dir: Vec2,
    pub speed: f32,
    #[inspect(proxy_type = "InspectDragf")]
    pub radius: f32,
    pub group: PhysicsGroup,
}

impl Default for PhysicsObject {
    fn default() -> Self {
        Self {
            dir: vec2!(1.0, 0.0),
            speed: 0.0,
            radius: 1.0,
            group: PhysicsGroup::Unknown,
        }
    }
}

pub type CollisionWorld = flat_spatial::DenseGrid<PhysicsObject>;

#[derive(Clone, Component, Debug)]
#[storage(VecStorage)]
pub struct Collider(pub DenseGridHandle);

impl InspectRenderDefault<Collider> for Collider {
    fn render(_: &[&Collider], _: &'static str, _: &mut World, _: &Ui, _: &InspectArgsDefault) {
        unimplemented!()
    }

    fn render_mut(
        data: &mut [&mut Collider],
        label: &'static str,
        world: &mut World,
        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!()
        }
        let d = &mut data[0];

        let mut obj = { *world.read_resource::<CollisionWorld>().get(d.0).unwrap().1 };

        let changed = <PhysicsObject as InspectRenderDefault<PhysicsObject>>::render_mut(
            &mut [&mut obj],
            label,
            world,
            ui,
            args,
        );

        let coworld: &mut CollisionWorld = &mut world.write_resource::<CollisionWorld>();
        if changed {
            *coworld.get_mut(d.0).unwrap().1 = obj;
        }
        changed
    }
}
