use legion::Entity;

#[derive(Default, Debug, Clone, Copy)]
pub struct InspectedEntity {
    pub e: Option<Entity>,
    pub dirty: bool, // Modified by inspection
    pub dist2: f32,
}
