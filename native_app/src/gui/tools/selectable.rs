use crate::gui::{InspectedBuilding, InspectedEntity, Tool};
use crate::inputmap::{InputAction, InputMap};
use crate::uiworld::UiWorld;
use geom::Vec2;
use simulation::map::ProjectFilter;
use simulation::{AnyEntity, Simulation};

pub fn select_radius(id: AnyEntity) -> f32 {
    match id {
        AnyEntity::VehicleID(_) => 5.0,
        AnyEntity::TrainID(_) => 10.0,
        AnyEntity::WagonID(_) => 10.0,
        AnyEntity::FreightStationID(_) => 0.0,
        AnyEntity::CompanyID(_) => 0.0,
        AnyEntity::HumanID(_) => 3.0,
    }
}

/// Selectable allows to select entities by clicking on them
pub fn selectable(sim: &Simulation, uiworld: &UiWorld) {
    profiling::scope!("gui::selectable");
    let mut inspected = uiworld.write::<InspectedEntity>();
    let mut inspected_b = uiworld.write::<InspectedBuilding>();
    let inp = uiworld.read::<InputMap>();
    let tool = uiworld.read::<Tool>();

    if inp.just_act.contains(&InputAction::Select)
        && matches!(*tool, Tool::Hand)
        && !inspected.dontclear
    {
        let unproj = unwrap_ret!(inp.unprojected);

        let w = sim.world();

        inspected.dist2 = f32::INFINITY;
        inspected.e = None;

        w.query_selectable_pos()
            .for_each(|(id, pos): (AnyEntity, Vec2)| {
                let dist2 = (pos - unproj.xy()).mag2();
                let rad = select_radius(id);
                if dist2 >= rad * rad || dist2 >= inspected.dist2 {
                    return;
                }
                inspected.dist2 = dist2;
                inspected.e = Some(id);
            });
    }

    if inp.just_act.contains(&InputAction::Select)
        && matches!(*tool, Tool::Hand)
        && !inspected_b.dontclear
    {
        inspected_b.e = None;
        if inspected.e.is_none() {
            let unproj = unwrap_ret!(inp.unprojected);
            let map = sim.map();
            inspected_b.e = map
                .spatial_map()
                .query(unproj.xy(), ProjectFilter::BUILDING)
                .find_map(|x| x.as_building());
        }
    }
    inspected.dontclear = false;
    inspected_b.dontclear = false;

    if let Some(e) = inspected.e {
        if !sim.world().contains(e) {
            inspected.e = None;
        }
    }

    if let Some(b) = inspected_b.e {
        if !sim.map().buildings().contains_key(b) {
            inspected_b.e = None;
        }
    }

    if inp.just_act.contains(&InputAction::Close) || matches!(*tool, Tool::Bulldozer) {
        inspected.e = None;
        inspected_b.e = None;
    }
}
