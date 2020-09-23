use egregoria::engine_interaction::{MouseInfo, RenderStats};
use egregoria::utils::frame_log::FrameLog;
use egregoria::Egregoria;
use imgui::{im_str, Ui};

pub fn info(ui: &Ui, goria: &mut Egregoria) {
    let stats = goria.read::<RenderStats>();
    let mouse = goria.read::<MouseInfo>().unprojected;

    ui.text("Averaged over last 10 frames: ");
    ui.text(im_str!(
        "World update time: {:.1}ms",
        stats.world_update.time_avg() * 1000.0
    ));
    ui.text(im_str!(
        "Render time: {:.1}ms",
        stats.render.time_avg() * 1000.0
    ));
    ui.text(im_str!(
        "Souls desires time: {:.1}ms",
        stats.souls_desires.time_avg() * 1000.0
    ));
    ui.text(im_str!(
        "Souls apply time: {:.1}ms",
        stats.souls_apply.time_avg() * 1000.0
    ));
    ui.text(im_str!("Mouse pos: {:.1} {:.1}", mouse.x, mouse.y));
    ui.separator();
    ui.text("Frame log");
    let flog = goria.read::<FrameLog>();
    {
        let fl = flog.get_frame_log();
        for s in &*fl {
            ui.text(im_str!("{}", s));
        }
    }
    flog.clear();
}
