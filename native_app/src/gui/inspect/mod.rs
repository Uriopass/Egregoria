use crate::gui::inspect::inspect_train::inspect_train;
use crate::gui::windows::debug::DebugState;
use crate::gui::{FollowEntity, InspectedBuilding, InspectedEntity};
use crate::uiworld::UiWorld;
use egui::{Context, Ui, Window};
use inspect_building::inspect_building;
use inspect_debug::InspectRenderer;
use inspect_human::inspect_human;
use inspect_vehicle::inspect_vehicle;
use simulation::map::BuildingID;
use simulation::{AnyEntity, Simulation};
use slotmapd::Key;

mod inspect_building;
mod inspect_debug;
mod inspect_human;
mod inspect_train;
mod inspect_vehicle;

pub fn inspector(ui: &Context, uiworld: &mut UiWorld, sim: &Simulation) {
    profiling::scope!("topgui::inspector");
    let inspected_building = *uiworld.read::<InspectedBuilding>();
    if let Some(b) = inspected_building.e {
        inspect_building(uiworld, sim, ui, b);
    }

    let e = unwrap_or!(uiworld.read::<InspectedEntity>().e, return);

    let force_debug_inspect = uiworld.read::<DebugState>().debug_inspector;

    let mut is_open = true;
    match e {
        AnyEntity::HumanID(id) if !force_debug_inspect => {
            is_open = inspect_human(uiworld, sim, ui, id);
        }
        AnyEntity::VehicleID(id) if !force_debug_inspect => {
            is_open = inspect_vehicle(uiworld, sim, ui, id);
        }
        AnyEntity::WagonID(id) if !force_debug_inspect => {
            let Some(w) = sim.world().get(id) else { return; };
            let train_id = w.itfollower.leader;
            uiworld.write::<InspectedEntity>().e = Some(AnyEntity::TrainID(train_id));
        }
        AnyEntity::TrainID(id) if !force_debug_inspect => {
            is_open = inspect_train(uiworld, sim, ui, id);
        }
        _ => {
            Window::new("Inspect")
                .default_size([400.0, 500.0])
                .default_pos([30.0, 160.0])
                .resizable(true)
                .open(&mut is_open)
                .show(ui, |ui| {
                    let mut ins = InspectRenderer { entity: e };
                    ins.render(uiworld, sim, ui);
                    uiworld.write::<InspectedEntity>().e = Some(ins.entity);
                });
        }
    }

    if !is_open {
        uiworld.write::<InspectedEntity>().e = None;
    }
}

pub fn building_link(uiworld: &mut UiWorld, sim: &Simulation, ui: &mut Ui, b: BuildingID) {
    if ui.link(format!("{:?}", b.data())).clicked() {
        uiworld.write::<InspectedBuilding>().e = Some(b);
        if let Some(b) = sim.map().buildings().get(b) {
            uiworld.camera_mut().targetpos = b.door_pos;
        }
    }
}

pub fn entity_link(uiworld: &mut UiWorld, sim: &Simulation, ui: &mut Ui, e: impl Into<AnyEntity>) {
    entity_link_inner(uiworld, sim, ui, e.into())
}

fn entity_link_inner(uiworld: &mut UiWorld, sim: &Simulation, ui: &mut Ui, e: AnyEntity) {
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

    if ui.link(linkname).clicked() {
        uiworld.write::<InspectedEntity>().e = Some(e);
        if sim.pos_any(e).is_some() {
            uiworld.write::<FollowEntity>().0 = Some(e);
        }
    }
}

pub fn follow_button(uiworld: &UiWorld, ui: &mut Ui, id: impl Into<AnyEntity>) {
    follow_button_inner(uiworld, ui, id.into())
}

fn follow_button_inner(uiworld: &UiWorld, ui: &mut Ui, id: AnyEntity) {
    let mut follow = uiworld.write::<FollowEntity>();
    if follow.0 != Some(id) && ui.small_button("follow").clicked() {
        follow.0 = Some(id);
    }
}
