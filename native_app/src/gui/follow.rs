use hecs::Entity;
register_resource_noserialize!(FollowEntity);
#[derive(Copy, Clone, Default)]
pub struct FollowEntity(pub Option<Entity>);
