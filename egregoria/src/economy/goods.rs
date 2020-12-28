use crate::economy::Market;
use legion::system;

pub struct Wheat;
pub struct Flour;
pub struct Bread;

#[system]
pub fn goods_market_update(#[resource] wheat: &mut Market<Wheat>) {
    for trade in wheat.make_trades() {
        log::info!("wheat trade {:?}", trade);
    }
}
