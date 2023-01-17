use crate::uiworld::UiWorld;
use egregoria::engine_interaction::WorldCommand;
use egregoria::Egregoria;
use egui::Context;

use egregoria::map::{BuildingID, Zone};
use egregoria::map_dynamic::BuildingInfos;
use egregoria::souls::goods_company::GoodsCompany;
use egui_inspect::{Inspect, InspectArgs, InspectVec2Rotation};

pub(crate) fn inspect_building(
    uiworld: &mut UiWorld,
    goria: &Egregoria,
    ui: &Context,
    id: BuildingID,
) {
    let map = goria.map();
    let Some(building) = map.buildings().get(id) else { return; };

    let owner = goria.read::<BuildingInfos>().owner(building.id);

    egui::Window::new("Building").show(ui, |ui| {
        ui.label(format!("{:?}", building.id));
        ui.label(format!("{:?}", building.kind));

        if let Some(ref zone) = building.zone {
            let mut cpy = zone.filldir;
            if InspectVec2Rotation::render_mut(&mut cpy, "fill dir", ui, &InspectArgs::default()) {
                uiworld.commands().push(WorldCommand::UpdateZone {
                    building: id,
                    zone: Zone {
                        filldir: cpy,
                        ..zone.clone()
                    },
                })
            }
        }

        let Some(soul) = owner else { return; };
        let Some(goods) = goria.comp::<GoodsCompany>(soul.0) else { return; };

        ui.label(format!("progress: {:?}", goods.progress));
    });
}
