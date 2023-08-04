use crate::game_loop::State;
use crate::inputmap::{InputAction, InputMap};
use egregoria::AnyEntity;
use egui::Ui;

/// FollowEntity is a component that tells the camera to follow an entity
/// Entity is defined by a function that returns the position of the entity
#[derive(Default)]
pub struct FollowEntity(pub Option<AnyEntity>);

impl FollowEntity {
    pub fn update_ui(&mut self, ui: &mut Ui, entity: AnyEntity) {
        if self.0.is_none() {
            if ui.small_button("Follow").clicked() {
                self.0.replace(entity);
            }
            return;
        }

        if ui.small_button("Unfollow").clicked() {
            self.0.take();
        }
    }

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
            if let Some(pos) = state.goria.read().unwrap().pos_any(e) {
                state.camera.follow(pos);
            }
        }
    }
}
