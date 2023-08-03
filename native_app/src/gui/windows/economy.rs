use crate::uiworld::UiWorld;
use common::timestep::UP_DT;
use egregoria::economy::{
    EcoStats, ItemHistories, ItemRegistry, Market, HISTORY_SIZE, LEVEL_FREQS, LEVEL_NAMES,
};
use egregoria::Egregoria;
use egui::plot::{Line, PlotPoints};
use egui::{Align2, Color32, Ui};
use geom::Color;
use slotmapd::Key;
use std::cmp::Reverse;
use std::collections::HashSet;

enum EconomyTab {
    ImportExports,
    InternalTrade,
    MarketPrices,
}

#[derive(Copy, Clone, Default)]
enum HistoryType {
    #[default]
    Money,
    Items,
}

struct EconomyState {
    pub curlevel: usize,
    pub tab: EconomyTab,
    pub hist_type: HistoryType,
}

/// Economy window
/// Shows the economy stats
pub(crate) fn economy(
    window: egui::Window<'_>,
    ui: &egui::Context,
    uiw: &mut UiWorld,
    goria: &Egregoria,
) {
    uiw.check_present(|| EconomyState {
        curlevel: 0,
        tab: EconomyTab::ImportExports,
        hist_type: Default::default(),
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
                if ui
                    .selectable_label(
                        matches!(state.tab, EconomyTab::MarketPrices),
                        "Market Prices",
                    )
                    .clicked()
                {
                    state.tab = EconomyTab::MarketPrices;
                }
            });

            ui.horizontal(|ui| {
                for (i, level) in LEVEL_NAMES.iter().enumerate() {
                    if ui.selectable_label(i == state.curlevel, *level).clicked() {
                        state.curlevel = i;
                    }
                }
                if matches!(state.tab, EconomyTab::ImportExports) {
                    ui.separator();
                    if ui
                        .selectable_label(matches!(state.hist_type, HistoryType::Money), "Money")
                        .clicked()
                    {
                        state.hist_type = HistoryType::Money
                    }
                    if ui
                        .selectable_label(matches!(state.hist_type, HistoryType::Items), "Items")
                        .clicked()
                    {
                        state.hist_type = HistoryType::Items;
                    }
                }
            });

            let seconds_per_step = LEVEL_FREQS[state.curlevel] as f64 * UP_DT.as_secs_f64();
            let xs: Vec<f64> = (0..HISTORY_SIZE)
                .map(|i| i as f64 * seconds_per_step)
                .collect();
            let EconomyState {
                curlevel,
                ref tab,
                hist_type,
            } = *state;
            let render_history = |ui: &mut Ui, history: &ItemHistories, hist_type: HistoryType| {
                let filterid = ui.id().with("filter");
                let mut filter = ui.data_mut(|d| {
                    d.get_temp_mut_or_insert_with(filterid, HashSet::new)
                        .clone()
                });
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
                        let cursor = history.cursors()[curlevel];
                        for (id, history) in history.iter_histories(curlevel) {
                            if !filter.is_empty() && !filter.contains(&id) {
                                continue;
                            }
                            let holder;
                            let ring = match hist_type {
                                HistoryType::Items => &history.past_ring_items,
                                HistoryType::Money => {
                                    holder = history.past_ring_money.map(|x| x.bucks().abs());
                                    &holder
                                }
                            };

                            let maxval = *ring.iter().max().unwrap();
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
                            let heights = ring[c_next..HISTORY_SIZE]
                                .iter()
                                .chain(ring[0..c_next].iter())
                                .copied()
                                .zip(xs.iter())
                                .map(|(v, x)| [*x, v as f64])
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
                        let mut overall_total = 0;
                        egui::Grid::new("ecogrid").show(ui, |ui| {
                            let mut histories: Vec<_> = history
                                .iter_histories(curlevel)
                                .map(|(id, level)| {
                                    (id, {
                                        match hist_type {
                                            HistoryType::Items => {
                                                level.past_ring_items.iter().sum::<i64>()
                                            }
                                            HistoryType::Money => level
                                                .past_ring_money
                                                .iter()
                                                .map(|x| x.bucks().abs())
                                                .sum::<i64>(),
                                        }
                                    })
                                })
                                .filter(|(_, x)| *x != 0)
                                .collect();
                            histories.sort_by_key(|(_, sum)| Reverse(*sum));

                            for (id, sum) in histories {
                                let iname = &registry[id].name;
                                let mut enabled = filter.contains(&id);
                                if ui.checkbox(&mut enabled, iname).changed() {
                                    if enabled {
                                        filter.insert(id);
                                    } else {
                                        filter.remove(&id);
                                    }
                                }
                                let suffix = match hist_type {
                                    HistoryType::Items => "",
                                    HistoryType::Money => "$",
                                };
                                ui.label(format!("{}{}", sum, suffix));
                                ui.end_row();
                                overall_total += sum;
                            }
                        });
                        if matches!(hist_type, HistoryType::Money) {
                            ui.separator();
                            ui.label(format!("Total: {}$", overall_total));
                        }
                    });
                ui.data_mut(move |d| {
                    d.insert_temp(filterid, filter);
                });
            };

            match tab {
                EconomyTab::ImportExports => {
                    let (label_left, label_right) = match hist_type {
                        HistoryType::Items => ("Imports", "Exports"),
                        HistoryType::Money => ("Expenses", "Income"),
                    };
                    ui.columns(2, |ui| {
                        ui[0].push_id(0, |ui| {
                            ui.label(label_left);
                            render_history(ui, &ecostats.imports, hist_type);
                        });
                        ui[1].push_id(1, |ui| {
                            ui.label(label_right);
                            render_history(ui, &ecostats.exports, hist_type);
                        });
                    });
                }
                EconomyTab::InternalTrade => {
                    ui.push_id(2, |ui| {
                        render_history(ui, &ecostats.internal_trade, HistoryType::Items);
                    });
                }
                EconomyTab::MarketPrices => {
                    ui.push_id(3, |ui| {
                        render_market_prices(goria, ui);
                    });
                }
            }
            ui.allocate_space(ui.available_size());
        });
}

fn render_market_prices(goria: &Egregoria, ui: &mut Ui) {
    let registry = goria.read::<ItemRegistry>();
    let market = goria.read::<Market>();
    egui::Grid::new("marketprices").show(ui, |ui| {
        for (id, market) in market.iter() {
            ui.label(&registry[*id].name);
            ui.label(market.ext_value.to_string());
            ui.end_row();
        }
    });
}
