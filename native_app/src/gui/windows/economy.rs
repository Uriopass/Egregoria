use crate::uiworld::UiWorld;
use egregoria::economy::{EcoStats, ItemHistories, ItemRegistry, Market, LEVEL_FREQS};
use egregoria::Egregoria;
use egui::Ui;
use geom::{vec2, Color, Vec2};
use slotmap::Key;

struct EconomyState {
    pub curlevel: usize,
}

pub(crate) fn economy(window: egui::Window<'_>, ui: &mut Ui, uiw: &mut UiWorld, goria: &Egregoria) {
    uiw.check_present(|| EconomyState { curlevel: 0 });
    let mut state = uiw.write::<EconomyState>();
    let market = goria.read::<Market>();
    let registry = goria.read::<ItemRegistry>();
    let ecostats = goria.read::<EcoStats>();
    let [w, h] = ui.io().display_size;

    window
        .position([w * 0.5, h * 0.5], Condition::Appearing)
        .position_pivot([0.5, 0.5])
        .size([600.0, h * 0.6], Condition::Appearing)
        .build(ui, || {
            let inner = market.inner();

            let r = ui.get_window_draw_list();
            let [wx, wy] = ui.window_pos();
            let [wh, _] = ui.window_size();
            let off = vec2(wx, wy);

            let draw_line = |start: Vec2, end: Vec2, c: Color| {
                r.add_line(
                    (off + start).into(),
                    (off + end).into(),
                    ImColor32::from_rgba(
                        (c.r * 255.0) as u8,
                        (c.g * 255.0) as u8,
                        (c.b * 255.0) as u8,
                        (c.a * 255.0) as u8,
                    ),
                )
                .build();
            };

            let draw_rect = |pos: Vec2, size: Vec2, c: Color| {
                r.add_rect(
                    (off + pos).into(),
                    (off + pos + size).into(),
                    ImColor32::from_rgba(
                        (c.r * 255.0) as u8,
                        (c.g * 255.0) as u8,
                        (c.b * 255.0) as u8,
                        (c.a * 255.0) as u8,
                    ),
                )
                .build();
            };

            if let Some(level) = egui::ComboBox::new("Level")
                .preview_value(format!("{}", state.curlevel))
                .begin(ui)
            {
                for (i, l) in LEVEL_FREQS.iter().enumerate() {
                    if egui::Selectable::new(format!("{}", l)).build(ui) {
                        state.curlevel = i;
                    }
                }
                level.end();
            }

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

            ui.dummy([0.0, tweak!(210.0)]);

            ui.columns(5, "Economy", false);

            ui.text("Commodity");
            ui.next_column();
            ui.text("Satisfaction");
            ui.next_column();
            ui.text("Offer");
            ui.next_column();
            ui.text("Demand");
            ui.next_column();
            ui.text("Capital");
            ui.next_column();

            for item in registry.iter() {
                let market = unwrap_or!(inner.get(&item.id), {
                    log::warn!("market does not exist for commodity {}", &item.name);
                    continue;
                });

                let buy = market.buy_orders();
                let sell = market.sell_orders();
                let capital = market.capital_map();
                let tot_capital = capital.values().sum::<i32>();
                let offer = sell.values().map(|x| x.1).sum::<i32>();
                let demand = buy.values().map(|x| x.1).sum::<i32>();

                if tot_capital == 0 && offer == 0 && demand == 0 {
                    continue;
                }

                let diff = offer - demand;

                ui.text(&item.label);
                ui.next_column();

                if diff == 0 {
                    ui.text_colored([0.8, 0.4, 0.2, 1.0], "±0");
                }
                if diff > 0 {
                    ui.text_colored([0.0, 1.0, 0.0, 1.0], format!("+{}", diff));
                }
                if diff < 0 {
                    ui.text_colored([1.0, 0.0, 0.0, 1.0], format!("{}", diff));
                }
                ui.next_column();

                ui.text(format!("{}", offer));
                ui.next_column();

                ui.text(format!("{}", demand));
                ui.next_column();

                ui.text(format!("{}", tot_capital));
                ui.next_column();
            }
        });
}
