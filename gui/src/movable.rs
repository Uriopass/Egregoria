use common::inspect::InspectedEntity;
use egregoria::engine_interaction::{MouseButton, MouseInfo, Movable, TimeInfo};
use egregoria::physics::Kinematics;
use geom::{Transform, Vec2};
use legion::world::SubWorld;
use legion::{system, EntityStore};

#[derive(Default)]
pub struct MovableSystem {
    clicked_at: Option<Vec2>,
}

#[system]
#[read_component(Movable)]
#[write_component(Transform)]
#[write_component(Kinematics)]
pub fn movable(
    #[state] sself: &mut MovableSystem,
    #[resource] mouse: &MouseInfo,
    #[resource] time: &TimeInfo,
    #[resource] inspected: &InspectedEntity,
    sw: &mut SubWorld,
) {
    if let Some(e) = inspected.e {
        let mut entry = sw.entry_mut(e).unwrap();

        if mouse.buttons.contains(&MouseButton::Left) && entry.get_component::<Movable>().is_ok() {
            match &mut sself.clicked_at {
                None => {
                    if let Ok(kin) = entry.get_component_mut::<Kinematics>() {
                        kin.velocity = Vec2::ZERO;
                    }
                    sself.clicked_at = Some(mouse.unprojected);
                }
                Some(off) => {
                    let p = entry
                        .get_component_mut::<Transform>()
                        .expect("Movable entity doesn't have a position");
                    let old_pos = p.position();
                    let new_pos = old_pos + (mouse.unprojected - *off);
                    *off = mouse.unprojected;
                    if new_pos != old_pos {
                        p.set_position(new_pos);
                        if let Ok(kin) = entry.get_component_mut::<Kinematics>() {
                            kin.velocity = Vec2::ZERO;
                        }
                    }
                }
            }
            return;
        }
    }

    if let Some(off) = sself.clicked_at.take() {
        if let Some(e) = inspected.e {
            let mut entry = sw.entry_mut(e).unwrap();
            if let Ok(kin) = entry.get_component_mut::<Kinematics>() {
                kin.velocity = (mouse.unprojected - off) / time.delta.max(1.0 / 30.0);
            }
        }
    }
}
