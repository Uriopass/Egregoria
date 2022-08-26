use crate::uiworld::UiWorld;
use egregoria::economy::{ItemRegistry, Market};
use egregoria::Egregoria;
use imgui::{Condition, Ui};

pub(crate) fn economy(
    window: imgui::Window<'_, &'static str>,
    ui: &Ui<'_>,
    _: &mut UiWorld,
    goria: &Egregoria,
) {
    let market = goria.read::<Market>();
    let registry = goria.read::<ItemRegistry>();
    let [w, h] = ui.io().display_size;

    window
        .position([w * 0.5, h * 0.5], Condition::Appearing)
        .position_pivot([0.5, 0.5])
        .size([600.0, h * 0.6], Condition::Appearing)
        .build(ui, || {
            let inner = market.inner();

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
