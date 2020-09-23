use egregoria::pedestrians::Pedestrian;
use egregoria::vehicles::Vehicle;
use egregoria::Egregoria;
use imgui::{im_str, Ui};
use legion::IntoQuery;
use map_model::Map;

pub fn map(ui: &Ui, goria: &mut Egregoria) {
    let mut map = goria.write::<Map>();

    if ui.small_button(im_str!("build houses")) {
        map.build_buildings();
    }

    if ui.small_button(im_str!("load Paris map")) {
        map.clear();
        map_model::load_parismap(&mut map);
        map.build_buildings();
    }

    if ui.small_button(im_str!("load test field")) {
        map.clear();
        map_model::load_testfield(&mut map);
        map.build_buildings();
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
}
