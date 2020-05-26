use crate::engine_interaction::{MouseButton, MouseInfo, TimeInfo};
use crate::geometry::Vec2;
use crate::interaction::InspectedEntity;
use crate::physics::{Kinematics, Transform};
use serde::{Deserialize, Serialize};
use specs::prelude::ResourceId;
use specs::shrev::EventChannel;
use specs::{
    Component, Entity, NullStorage, Read, ReadStorage, System, SystemData, World, Write,
    WriteStorage,
};

#[derive(Component, Default, Clone, Serialize, Deserialize)]
#[storage(NullStorage)]
pub struct Movable;
empty_inspect_impl!(Movable);

#[derive(Debug)]
pub struct MovedEvent {
    pub entity: Entity,
    pub new_pos: Vec2,
    pub delta_pos: Vec2,
}

#[derive(Default)]
pub struct MovableSystem {
    clicked_at: Option<Vec2>,
}

#[derive(SystemData)]
pub struct MovableSystemData<'a> {
    mouse: Read<'a, MouseInfo>,
    time: Read<'a, TimeInfo>,
    inspected: Read<'a, InspectedEntity>,
    moved: Write<'a, EventChannel<MovedEvent>>,
    transforms: WriteStorage<'a, Transform>,
    kinematics: WriteStorage<'a, Kinematics>,
    movable: ReadStorage<'a, Movable>,
}

impl<'a> System<'a> for MovableSystem {
    type SystemData = MovableSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        if data.mouse.buttons.contains(&MouseButton::Left)
            && data
                .inspected
                .e
                .map_or(false, |e| data.movable.get(e).is_some())
        {
            let e = data.inspected.e.unwrap();
            match &mut self.clicked_at {
                None => {
                    if let Some(kin) = data.kinematics.get_mut(e) {
                        kin.velocity = Vec2::zero();
                        kin.acceleration = Vec2::zero();
                    }
                    self.clicked_at = Some(data.mouse.unprojected);
                }
                Some(off) => {
                    let p = data.transforms.get_mut(e).unwrap();
                    let old_pos = p.position();
                    let new_pos = old_pos + (data.mouse.unprojected - *off);
                    *off = data.mouse.unprojected;
                    if new_pos != old_pos {
                        if let Some(kin) = data.kinematics.get_mut(e) {
                            kin.velocity = Vec2::zero();
                            kin.acceleration = Vec2::zero();
                        }
                        p.set_position(new_pos);
                        data.moved.single_write(MovedEvent {
                            entity: e,
                            new_pos,
                            delta_pos: new_pos - old_pos,
                        });
                    }
                }
            }
        } else if let Some(off) = self.clicked_at.take() {
            if let Some(e) = data.inspected.e {
                if let Some(kin) = data.kinematics.get_mut(e) {
                    kin.velocity = (data.mouse.unprojected - off) / data.time.delta.max(1.0 / 30.0);
                }
            }
        }
    }
}
