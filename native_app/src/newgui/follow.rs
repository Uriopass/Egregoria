use crate::game_loop::State;
use crate::inputmap::{InputAction, InputMap};
use simulation::AnyEntity;

/// FollowEntity is a component that tells the camera to follow an entity
#[derive(Default)]
pub struct FollowEntity(pub Option<AnyEntity>);

impl FollowEntity {
    pub fn update_camera(state: &mut State) {
        let just = &state.uiw.read::<InputMap>().just_act;
        if [
            InputAction::Close,
            InputAction::CameraMove,
            InputAction::GoForward,
            InputAction::GoBackward,
            InputAction::GoLeft,
            InputAction::GoRight,
        ]
        .iter()
        .any(|x| just.contains(x))
        {
            state.uiw.write::<FollowEntity>().0.take();
        }

        if let Some(e) = state.uiw.read::<FollowEntity>().0 {
            if let Some(pos) = state.sim.read().unwrap().pos_any(e) {
                state.uiw.camera_mut().follow(pos);
            }
        }
    }
}
