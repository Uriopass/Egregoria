use egregoria::AnyEntity;
use egui::Ui;

/// FollowEntity is a component that tells the camera to follow an entity
/// Entity is defined by a function that returns the position of the entity
#[derive(Default)]
pub struct FollowEntity(pub(crate) Option<AnyEntity>);

impl FollowEntity {
    pub fn update(&mut self, ui: &mut Ui, entity: AnyEntity) {
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
}
