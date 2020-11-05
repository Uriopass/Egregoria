use legion::Entity;

#[derive(Copy, Clone, Default, Debug)]
pub struct InspectedEntity {
    pub e: Option<Entity>,
    pub dirty: bool, // Modified by inspection
    pub dist2: f32,
}
