use crate::uiworld::UiWorld;
use common::timestep::UP_DT;
use egregoria::economy::{
    EcoStats, ItemHistories, ItemRegistry, HISTORY_SIZE, LEVEL_FREQS, LEVEL_NAMES,
};
use egregoria::Egregoria;
use egui::plot::{Line, PlotPoints};
use egui::{Align2, Color32, Ui};
use geom::Color;
use slotmap::Key;
use std::cmp::Reverse;

enum EconomyTab {
    ImportExports,
    InternalTrade,
}

struct EconomyState {
    pub curlevel: usize,
    pub tab: EconomyTab,
}

pub(crate) fn economy(
    window: egui::Window<'_>,
    ui: &egui::Context,
    uiw: &mut UiWorld,
    goria: &Egregoria,
) {
    uiw.check_present(|| EconomyState {
        curlevel: 0,
        tab: EconomyTab::ImportExports,
    });
    let mut state = uiw.write::<EconomyState>();
    let ecostats = goria.read::<EcoStats>();
    let registry = goria.read::<ItemRegistry>();

    window
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size([700.0, 500.0])
        .show(ui, move |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(
                        matches!(state.tab, EconomyTab::ImportExports),
                        "Import/Exports",
                    )
                    .clicked()
                {
                    state.tab = EconomyTab::ImportExports;
                }
                if ui
                    .selectable_label(
                        matches!(state.tab, EconomyTab::InternalTrade),
                        "Internal Trade",
                    )
                    .clicked()
                {
                    state.tab = EconomyTab::InternalTrade;
                }
            });

            ui.horizontal(|ui| {
                for (i, level) in LEVEL_NAMES.iter().enumerate() {
                    if ui.selectable_label(i == state.curlevel, *level).clicked() {
                        state.curlevel = i;
                    }
                }
            });

            let seconds_per_step = LEVEL_FREQS[state.curlevel] as f64 * UP_DT.as_secs_f64();
            let xs: Vec<f64> = (0..HISTORY_SIZE)
                .map(|i| i as f64 * seconds_per_step)
                .collect();
            let render_history = |ui: &mut Ui, history: &ItemHistories| {
                egui::plot::Plot::new("ecoplot")
                    .height(200.0)
                    .allow_boxed_zoom(false)
                    .include_y(0.0)
                    .include_x(0.0)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .allow_zoom(false)
                    .show(ui, |ui| {
                        let mut overallmax = 0;
                        let cursor = history.cursors()[state.curlevel];
                        for (id, history) in history.iter_histories(state.curlevel) {
                            let maxval = *history.past_ring.iter().max().unwrap();
                            if maxval == 0 {
                                continue;
                            }
                            if maxval > overallmax {
                                overallmax = maxval;
                            }

                            let h = common::hash_u64(id.data().as_ffi());
                            let random_col = Color::new(
                                0.5 + 0.5 * common::rand::rand2(h as f32, 0.0),
                                0.5 + 0.5 * common::rand::rand2(h as f32, 1.0),
                                0.5 + 0.5 * common::rand::rand2(h as f32, 2.0),
                                1.0,
                            );

                            let c_next = (cursor + 1) % HISTORY_SIZE;

                            let mut first_zeros = false;
                            let heights = history.past_ring[c_next..HISTORY_SIZE]
                                .iter()
                                .chain(history.past_ring[0..c_next].iter())
                                .copied()
                                .zip(xs.iter())
                                .map(|(v, x)| [*x as f64, v as f64])
                                .filter(|[_, y]| {
                                    if !first_zeros && *y > 0.0 {
                                        first_zeros = true;
                                    }
                                    first_zeros
                                });

                            let iname = &registry[id].name;

                            ui.line(
                                Line::new(PlotPoints::from_iter(heights))
                                    .color(Color32::from_rgba_unmultiplied(
                                        (random_col.r * 255.0) as u8,
                                        (random_col.g * 255.0) as u8,
                                        (random_col.b * 255.0) as u8,
                                        (random_col.a * 255.0) as u8,
                                    ))
                                    .name(iname),
                            );
                        }
                        ui.line(
                            Line::new(
                                [
                                    [0.0, 0.0],
                                    [
                                        HISTORY_SIZE as f64 * seconds_per_step,
                                        1.0 + 1.25 * overallmax as f64,
                                    ],
                                ]
                                .into_iter()
                                .collect::<PlotPoints>(),
                            )
                            .width(0.001)
                            .color(Color32::from_white_alpha(1)),
                        );
                    });

                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .vscroll(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(ui.available_width());
                        });
                        egui::Grid::new("ecogrid").show(ui, |ui| {
                            let mut histories: Vec<_> = history
                                .iter_histories(state.curlevel)
                                .map(|(id, level)| (id, level.past_ring.iter().sum::<u32>()))
                                .filter(|(_, x)| *x > 0)
                                .collect();
                            histories.sort_by_key(|(_, sum)| Reverse(*sum));

                            for (id, sum) in histories {
                                let iname = &registry[id].name;
                                ui.label(iname);
                                ui.label(format!("{}", sum));
                                ui.end_row();
                            }
                        });
                    });
            };

            match state.tab {
                EconomyTab::ImportExports => {
                    ui.columns(2, |ui| {
                        ui[0].push_id(0, |ui| {
                            ui.label("Imports");
                            render_history(ui, &ecostats.imports);
                        });
                        ui[1].push_id(1, |ui| {
                            ui.label("Exports");
                            render_history(ui, &ecostats.exports);
                        });
                    });
                }
                EconomyTab::InternalTrade => {
                    render_history(ui, &ecostats.internal_trade);
                }
            }
            ui.allocate_space(ui.available_size());
        });
}
