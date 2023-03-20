use hecs::Entity;

/// FollowEntity is a component that tells the camera to follow an entity
#[derive(Copy, Clone, Default)]
pub(crate) struct FollowEntity(pub(crate) Option<Entity>);
