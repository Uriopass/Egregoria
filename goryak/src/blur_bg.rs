use yakui_core::geometry::Color;
use yakui_widgets::colored_box_container;
use yakui_widgets::widgets::CutOut;
lazy_static::lazy_static! {
    pub static ref BLUR_TEXTURE: std::sync::Arc<std::sync::Mutex<Option<yakui_core::TextureId>>> = Default::default();
}

pub fn blur_bg(overlay_color: Color, children: impl FnOnce()) {
    let id = BLUR_TEXTURE.lock().unwrap().clone();
    let Some(id) = id else {
        colored_box_container(overlay_color, children);
        return;
    };

    CutOut::new(id, overlay_color).show_children(children);
}
