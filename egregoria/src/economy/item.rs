use crate::economy::Money;
use common::saveload::Encoder;
use common::FastMap;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};
use std::ops::Index;

#[derive(Serialize, Deserialize)]
struct ItemDefinition {
    name: String,
    label: String,
    #[serde(default)]
    ext_value: Money,
    #[serde(default)]
    transport_cost: Money,
    #[serde(default)]
    optout_exttrade: bool,
}

new_key_type! {
    pub struct ItemID;
}

debug_inspect_impl!(ItemID);

#[derive(Default, Serialize, Deserialize)]
pub struct ItemRegistry {
    items: SlotMap<ItemID, Item>,
    item_names: FastMap<String, ItemID>,
}

impl Index<ItemID> for ItemRegistry {
    type Output = Item;
    fn index(&self, index: ItemID) -> &Self::Output {
        &self.items[index]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub id: ItemID,
    pub name: String,
    pub label: String,
    pub ext_value: Money,
    pub transport_cost: Money,
    pub optout_exttrade: bool,
}

impl ItemRegistry {
    pub fn id(&self, name: &str) -> ItemID {
        self.item_names
            .get(name)
            .copied()
            .unwrap_or_else(|| panic!("no item in registry named {}", name))
    }

    pub fn try_id(&self, name: &str) -> Option<ItemID> {
        self.item_names.get(name).copied()
    }

    pub fn get(&self, id: ItemID) -> Option<&Item> {
        self.items.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ Item> + '_ {
        self.items.values()
    }

    pub fn load_item_definitions(&mut self, source: &str) {
        let definitions: Vec<ItemDefinition> = match common::saveload::JSON::decode(source.as_ref())
        {
            Ok(x) => x,
            Err(e) => {
                log::error!("error loading item definitions: {}", e);
                return;
            }
        };
        for definition in definitions {
            let name = definition.name.clone();
            let id = self.items.insert_with_key(move |id| Item {
                id,
                name: definition.name,
                label: definition.label,
                ext_value: definition.ext_value,
                transport_cost: definition.transport_cost,
                optout_exttrade: definition.optout_exttrade,
            });
            self.item_names.insert(name, id);
            log::info!("loaded {:?}", &self.items[id]);
        }
    }
}
