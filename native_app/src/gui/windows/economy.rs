use egregoria::economy::Market;
use egregoria::Egregoria;
use imgui::{Condition, Ui};

pub fn economy(window: imgui::Window, ui: &Ui, goria: &mut Egregoria) {
    let market = goria.read::<Market>();
    let [w, h] = ui.io().display_size;

    window
        .position([w * 0.5, h * 0.5], Condition::FirstUseEver)
        .position_pivot([0.5, 0.5])
        .size([600.0, h * 0.6], Condition::FirstUseEver)
        .build(ui, || {
            let inner = market.inner();
            for (kind, market) in inner {
                let buy = market.buy_orders();
                let sell = market.sell_orders();
                let capital = market.capital_map();
                let tot_capital = capital.values().sum::<i32>();
                if tot_capital == 0 {
                    continue;
                }

                ui.text(format!("{}", kind));
                ui.separator();
                ui.text(format!(
                    "offer: {}",
                    sell.values().map(|x| x.1).sum::<i32>()
                ));
                ui.text(format!(
                    "demand: {}",
                    buy.values().map(|x| x.1).sum::<i32>()
                ));
                ui.new_line();
            }
        });
}
