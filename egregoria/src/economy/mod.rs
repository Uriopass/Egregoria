mod goods;
mod jobs;
mod market;

pub use goods::*;
pub use jobs::*;
pub use market::*;

pub trait Commodity {}
impl<T> Commodity for T {}

pub trait CommodityList {}
