use crate::debug_gui::debug_window::DebugState;
use crate::gui::follow::FollowEntity;
use crate::gui::{InspectedBuilding, InspectedEntity};
use crate::uiworld::UiWorld;
use goryak::{button_primary, primary_link};
use inspect_building::inspect_building;
use inspect_human::inspect_human;
use inspect_train::inspect_train;
use inspect_vehicle::inspect_vehicle;
use simulation::map::BuildingID;
use simulation::{AnyEntity, Simulation};
use slotmapd::Key;

mod inspect_building;
mod inspect_human;
mod inspect_train;
mod inspect_vehicle;

pub fn new_inspector(uiworld: &UiWorld, sim: &Simulation) {
    profiling::scope!("hud::inspector");
    let inspected_building = *uiworld.read::<InspectedBuilding>();
    if let Some(b) = inspected_building.e {
        let is_open = inspect_building(uiworld, sim, b);
        if !is_open {
            uiworld.write::<InspectedBuilding>().e = None;
        }
    }

    let e = unwrap_or!(uiworld.read::<InspectedEntity>().e, return);

    let force_debug_inspect = uiworld.read::<DebugState>().debug_inspector;
    if force_debug_inspect {
        return;
    }

    let mut is_open = true;
    match e {
        AnyEntity::HumanID(id) if !force_debug_inspect => {
            is_open = inspect_human(uiworld, sim, id);
        }
        AnyEntity::VehicleID(id) if !force_debug_inspect => {
            is_open = inspect_vehicle(uiworld, sim, id);
        }
        AnyEntity::WagonID(id) if !force_debug_inspect => {
            let Some(w) = sim.world().get(id) else {
                return;
            };
            let train_id = w.itfollower.leader;
            uiworld.write::<InspectedEntity>().e = Some(AnyEntity::TrainID(train_id));
        }
        AnyEntity::TrainID(id) if !force_debug_inspect => {
            is_open = inspect_train(uiworld, sim, id);
        }
        _ => {}
    }

    if !is_open {
        uiworld.write::<InspectedEntity>().e = None;
    }
}

pub fn building_link(uiworld: &UiWorld, sim: &Simulation, b: BuildingID) {
    if primary_link(format!("{:?}", b.data())) {
        uiworld.write::<InspectedBuilding>().e = Some(b);
        if let Some(b) = sim.map().buildings().get(b) {
            uiworld.camera_mut().targetpos = b.door_pos;
        }
    }
}

pub fn entity_link(uiworld: &UiWorld, sim: &Simulation, e: impl Into<AnyEntity>) {
    entity_link_inner(uiworld, sim, e.into())
}

fn entity_link_inner(uiworld: &UiWorld, sim: &Simulation, e: AnyEntity) {
    let linkname = match e {
        AnyEntity::HumanID(id) => {
            if let Some(human) = sim.world().humans.get(id) {
                human.personal_info.name.to_string()
            } else {
                "???".to_string()
            }
        }
        _ => format!("{}", e),
    };

    if primary_link(linkname) {
        uiworld.write::<InspectedEntity>().e = Some(e);
        if sim.pos_any(e).is_some() {
            uiworld.write::<FollowEntity>().0 = Some(e);
        }
    }
}

pub fn follow_button(uiworld: &UiWorld, id: impl Into<AnyEntity>) {
    follow_button_inner(uiworld, id.into())
}

fn follow_button_inner(uiworld: &UiWorld, id: AnyEntity) {
    let mut follow = uiworld.write::<FollowEntity>();
    if follow.0 != Some(id) && button_primary("follow").show().clicked {
        follow.0 = Some(id);
    }
}
