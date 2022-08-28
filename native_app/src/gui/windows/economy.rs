use crate::uiworld::UiWorld;
use egregoria::economy::{EcoStats, ItemHistories, LEVEL_FREQS};
use egregoria::Egregoria;
use egui::{Align2, Color32, Rect, Rounding, Stroke};
use geom::{vec2, Color, Vec2};
use slotmap::Key;

struct EconomyState {
    pub curlevel: usize,
}

pub(crate) fn economy(
    window: egui::Window<'_>,
    ui: &egui::Context,
    uiw: &mut UiWorld,
    goria: &Egregoria,
) {
    uiw.check_present(|| EconomyState { curlevel: 0 });
    let mut state = uiw.write::<EconomyState>();
    let ecostats = goria.read::<EcoStats>();
    let [w, h]: [f32; 2] = ui.available_rect().size().into();

    window
        .default_pos([w * 0.5, h * 0.5])
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_size([600.0, h * 0.6])
        .show(ui, move |ui| {
            egui::ComboBox::from_label("Level").show_index(
                ui,
                &mut state.curlevel,
                LEVEL_FREQS.len(),
                |i| LEVEL_FREQS[i].to_string(),
            );

            let r = ui.painter();
            let [wh, _]: [f32; 2] = ui.available_size().into();

            let draw_line = |start: Vec2, end: Vec2, c: Color| {
                r.line_segment(
                    [[start.x, start.y].into(), [end.x, end.y].into()],
                    Stroke::new(
                        1.0,
                        Color32::from_rgba_unmultiplied(
                            (c.r * 255.0) as u8,
                            (c.g * 255.0) as u8,
                            (c.b * 255.0) as u8,
                            (c.a * 255.0) as u8,
                        ),
                    ),
                );
            };

            let draw_rect = |pos: Vec2, size: Vec2, c: Color| {
                r.rect_stroke(
                    Rect::from_min_size([pos.x, pos.y].into(), [size.x, size.y].into()),
                    Rounding::none(),
                    Stroke::new(
                        1.0,
                        Color32::from_rgba_unmultiplied(
                            (c.r * 255.0) as u8,
                            (c.g * 255.0) as u8,
                            (c.b * 255.0) as u8,
                            (c.a * 255.0) as u8,
                        ),
                    ),
                )
            };

            let render_history = |history: &ItemHistories, offx, offy, width, height| {
                const PADDING: f32 = 5.0;
                draw_rect(vec2(offx, offy), vec2(width, height), Color::gray(0.5));
                for (id, history) in history.iter_histories(state.curlevel) {
                    let h = common::hash_u64(id.data().as_ffi());
                    let random_col = Color::new(
                        common::rand::rand2(h as f32, 0.0),
                        common::rand::rand2(h as f32, 1.0),
                        common::rand::rand2(h as f32, 2.0),
                        1.0,
                    );

                    let maxval = history.past_ring.iter().copied().max().unwrap() as f32;

                    let heights = history
                        .past_ring
                        .iter()
                        .copied()
                        .map(|v| height - (height - PADDING * 2.0) * v as f32 / maxval);

                    let step = (width - PADDING * 2.0) / (heights.len() as f32);
                    for (x, (a, b)) in heights.clone().zip(heights.skip(1)).enumerate() {
                        let x = x as f32;
                        // Draw line from a to b
                        let a = Vec2::new(PADDING + offx + x * step, offy + a - PADDING);
                        let b = Vec2::new(PADDING + offx + (x + 1.0) * step, offy + b - PADDING);
                        draw_line(a, b, random_col);
                    }
                }
            };

            render_history(
                &ecostats.imports,
                tweak!(10.0),
                tweak!(70.0),
                wh * 0.5 - tweak!(12.0),
                tweak!(95.0),
            );

            render_history(
                &ecostats.exports,
                wh * 0.5 + tweak!(2.0),
                tweak!(70.0),
                wh * 0.5 - tweak!(12.0),
                tweak!(95.0),
            );

            ui.add_space(tweak!(210.0));
        });
}
