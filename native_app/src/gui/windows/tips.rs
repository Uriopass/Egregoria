use egregoria::Egregoria;
use imgui::{im_str, Ui};

pub fn tips(ui: &Ui, _goria: &mut Egregoria) {
    ui.text(im_str!("Select: Left click"));
    ui.text(im_str!("Move: Left drag"));
    ui.text(im_str!("Deselect: Escape"));
    ui.text(im_str!("Pan: Right click or Arrow keys"));
}
