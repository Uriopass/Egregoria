use crate::economy::Market;
use crate::rendering::immediate::ImmediateDraw;
use crate::SoulID;
use geom::Color;
use legion::world::SubWorld;
use legion::{system, EntityStore};

pub struct JobApplication;

pub struct Workers(pub Vec<SoulID>);

#[system]
#[write_component(Workers)]
pub fn job_market_update(
    #[resource] jobs: &mut Market<JobApplication>,
    #[resource] draw: &mut ImmediateDraw,
    subworld: &mut SubWorld,
) {
    for trade in jobs.make_trades() {
        subworld
            .entry_mut(trade.seller.0)
            .unwrap()
            .get_component_mut::<Workers>()
            .unwrap()
            .0
            .push(trade.buyer);

        draw.line(trade.buy_pos, trade.sell_pos, 1.0)
            .color(Color::CYAN)
            .persistent();
    }
}
