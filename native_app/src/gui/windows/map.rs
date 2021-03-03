use egregoria::map_dynamic::BuildingInfos;
use egregoria::pedestrians::Pedestrian;
use egregoria::vehicles::Vehicle;
use egregoria::Egregoria;
use imgui::{im_str, Ui};
use legion::IntoQuery;
use map_model::Map;

pub fn map(window: imgui::Window, ui: &Ui, goria: &mut Egregoria) {
    window.build(ui, || {
        let mut map = goria.write::<Map>();

        if ui.small_button(im_str!("build houses")) {
            let mut infos = goria.write::<BuildingInfos>();
            for build in map.build_buildings() {
                infos.insert(build);
            }
        }

        if ui.small_button(im_str!("load Paris map")) {
            map.clear();
            map_model::procgen::load_parismap(&mut map);
        }

        if ui.small_button(im_str!("load test field")) {
            map.clear();
            map_model::procgen::load_testfield(&mut map);
        }

        if ui.small_button(im_str!("clear the map")) {
            map.clear();
        }

        ui.text(im_str!(
            "{} pedestrians",
            <&Pedestrian>::query().iter(&goria.world).count()
        ));
        ui.text(im_str!(
            "{} vehicles",
            <&Vehicle>::query().iter(&goria.world).count()
        ));
    })
}
