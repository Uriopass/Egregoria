use hecs::Entity;
#[derive(Copy, Clone, Default)]
pub(crate) struct FollowEntity(pub(crate) Option<Entity>);
