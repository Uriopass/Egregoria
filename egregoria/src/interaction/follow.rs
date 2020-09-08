use legion::Entity;

#[derive(Default, Clone, Copy)]
pub struct FollowEntity(pub Option<Entity>);
