use crate::uiworld::UiWorld;
use egregoria::pedestrians::Pedestrian;
use egregoria::vehicles::Vehicle;
use egregoria::Egregoria;
use imgui::{im_str, Ui};
use legion::IntoQuery;

pub fn map(window: imgui::Window, ui: &Ui, uiworld: &mut UiWorld, goria: &Egregoria) {
    window.build(ui, || {
        if ui.small_button(im_str!("load Paris map")) {
            uiworld.commands().map_load_paris();
        }

        if ui.small_button(im_str!("load test field")) {
            uiworld.commands().map_load_testfield();
        }

        if ui.small_button(im_str!("clear the map")) {
            uiworld.commands().map_clear();
        }

        ui.text(im_str!(
            "{} pedestrians",
            <&Pedestrian>::query().iter(goria.world()).count()
        ));
        ui.text(im_str!(
            "{} vehicles",
            <&Vehicle>::query().iter(goria.world()).count()
        ));
    })
}
