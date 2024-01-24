use std::sync::atomic::AtomicU64;
use yakui_core::geometry::Color;
use yakui_core::TextureId;
use yakui_widgets::colored_box_container;
use yakui_widgets::widgets::CutOut;

static BLUR_TEXTURE: AtomicU64 = AtomicU64::new(u64::MAX);

pub fn set_blur_texture(tex: TextureId) {
    match tex {
        TextureId::Managed(x) => {
            panic!("Cannot use a managed texture as a blur texture: {:?}", x);
        }
        TextureId::User(id) => BLUR_TEXTURE.store(id, std::sync::atomic::Ordering::SeqCst),
    }
}

pub fn blur_texture() -> Option<TextureId> {
    let id = BLUR_TEXTURE.load(std::sync::atomic::Ordering::Relaxed);
    if id == u64::MAX {
        None
    } else {
        Some(TextureId::User(id))
    }
}

pub fn blur_bg(overlay_color: Color, radius: f32, children: impl FnOnce()) {
    let id = blur_texture();
    let Some(id) = id else {
        colored_box_container(overlay_color, children);
        return;
    };

    let mut c = CutOut::new(id, overlay_color);
    c.radius = radius;
    c.show_children(children);
}
