use crate::gui::inspect::inspect_train::inspect_train;
use crate::gui::windows::debug::DebugState;
use crate::gui::{InspectedBuilding, InspectedEntity};
use crate::uiworld::UiWorld;
use egregoria::map::BuildingID;
use egregoria::{AnyEntity, Egregoria};
use egui::{Context, Ui, Window};
use inspect_building::inspect_building;
use inspect_debug::InspectRenderer;
use inspect_human::inspect_human;
use inspect_vehicle::inspect_vehicle;
use slotmapd::Key;

mod inspect_building;
mod inspect_debug;
mod inspect_human;
mod inspect_train;
mod inspect_vehicle;

pub fn inspector(ui: &Context, uiworld: &mut UiWorld, goria: &Egregoria) {
    profiling::scope!("topgui::inspector");
    let inspected_building = *uiworld.read::<InspectedBuilding>();
    if let Some(b) = inspected_building.e {
        inspect_building(uiworld, goria, ui, b);
    }

    let e = unwrap_or!(uiworld.read::<InspectedEntity>().e, return);

    let force_debug_inspect = uiworld.read::<DebugState>().debug_inspector;

    let mut is_open = true;
    match e {
        AnyEntity::HumanID(id) if !force_debug_inspect => {
            is_open = inspect_human(uiworld, goria, ui, id);
        }
        AnyEntity::VehicleID(id) if !force_debug_inspect => {
            is_open = inspect_vehicle(uiworld, goria, ui, id);
        }
        AnyEntity::WagonID(id) if !force_debug_inspect => {
            let Some(w) = goria.world().get(id) else { return; };
            let train_id = w.itfollower.leader;
            is_open = inspect_train(uiworld, goria, ui, train_id);
        }
        AnyEntity::TrainID(id) if !force_debug_inspect => {
            is_open = inspect_train(uiworld, goria, ui, id);
        }
        _ => {
            Window::new("Inspect")
                .default_size([400.0, 500.0])
                .default_pos([30.0, 160.0])
                .resizable(true)
                .open(&mut is_open)
                .show(ui, |ui| {
                    let mut ins = InspectRenderer { entity: e };
                    ins.render(uiworld, goria, ui);
                    uiworld.write::<InspectedEntity>().e = Some(ins.entity);
                });
        }
    }

    if !is_open {
        uiworld.write::<InspectedEntity>().e = None;
    }
}

pub fn building_link(uiworld: &mut UiWorld, goria: &Egregoria, ui: &mut Ui, b: BuildingID) {
    if ui.link(format!("{:?}", b.data())).clicked() {
        uiworld.write::<InspectedBuilding>().e = Some(b);
        if let Some(b) = goria.map().buildings().get(b) {
            uiworld.camera_mut().targetpos = b.door_pos;
        }
    }
}

pub fn entity_link(uiworld: &mut UiWorld, goria: &Egregoria, ui: &mut Ui, e: impl Into<AnyEntity>) {
    entity_link_inner(uiworld, goria, ui, e.into())
}

fn entity_link_inner(uiworld: &mut UiWorld, goria: &Egregoria, ui: &mut Ui, e: AnyEntity) {
    let linkname = match e {
        AnyEntity::HumanID(id) => {
            if let Some(human) = goria.world().humans.get(id) {
                human.personal_info.name.to_string()
            } else {
                "???".to_string()
            }
        }
        _ => format!("{}", e),
    };

    if ui.link(linkname).clicked() {
        uiworld.write::<InspectedEntity>().e = Some(e);
        if let Some(pos) = goria.pos_any(e) {
            uiworld.camera_mut().targetpos = pos
        }
    }
}
