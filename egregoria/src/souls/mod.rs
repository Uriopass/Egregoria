use legion::Entity;
use map_model::BuildingID;
use slotmap::DenseSlotMap;

slotmap::new_key_type! {
    pub struct SoulID;
}

pub type Souls = DenseSlotMap<SoulID, Soul>;

#[derive(Clone)]
pub struct Soul {
    id: SoulID,
    car: Option<Entity>,
    home: BuildingID,
    work: BuildingID,
}
